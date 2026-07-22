use std::{
    fmt::{Display, Formatter},
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use sqlx::SqlitePool;
use tokio::{fs, sync::Semaphore};

use crate::{
    docker::{BuildRequest, DockerClient},
    git::{CloneRequest, GitClient},
};

use super::{
    model::{Application, ApplicationStatus},
    repository,
};

#[derive(Clone)]
pub(crate) struct DeploymentPreparer {
    database: SqlitePool,
    workspace_root: Arc<PathBuf>,
    git_client: Arc<dyn GitClient>,
    docker_client: Arc<dyn DockerClient>,
    concurrency: Arc<Semaphore>,
}

impl DeploymentPreparer {
    pub(crate) fn new(
        database: SqlitePool,
        workspace_root: PathBuf,
        git_client: Arc<dyn GitClient>,
        docker_client: Arc<dyn DockerClient>,
    ) -> Self {
        Self {
            database,
            workspace_root: Arc::new(workspace_root),
            git_client,
            docker_client,
            concurrency: Arc::new(Semaphore::new(1)),
        }
    }

    pub(crate) async fn prepare(&self, application: Application) {
        let _permit = self
            .concurrency
            .acquire()
            .await
            .expect("source preparation semaphore should remain open");

        if let Err(error) = self.prepare_deployment(&application).await {
            tracing::error!(application_id = %application.id, %error, "deployment preparation failed");

            if let Err(log_error) = repository::append_log(
                &self.database,
                application.id,
                error.stage(),
                "system",
                &error.to_string(),
            )
            .await
            {
                tracing::error!(application_id = %application.id, %log_error, "failed to persist deployment error log");
            }

            if let Err(status_error) =
                repository::mark_failed(&self.database, application.id, &error.to_string()).await
            {
                tracing::error!(application_id = %application.id, %status_error, "failed to persist failed status");
            }
        }
    }

    async fn prepare_deployment(&self, application: &Application) -> Result<(), DeploymentError> {
        let build_context = self.prepare_source(application).await?;
        self.build_image(application, build_context).await
    }

    async fn prepare_source(&self, application: &Application) -> Result<PathBuf, DeploymentError> {
        repository::transition_status(
            &self.database,
            application.id,
            ApplicationStatus::Queued,
            ApplicationStatus::Cloning,
        )
        .await?;
        repository::append_log(
            &self.database,
            application.id,
            "source",
            "system",
            "starting Git clone",
        )
        .await?;

        fs::create_dir_all(self.workspace_root.as_ref()).await?;
        let repository_directory = self.workspace_root.join(application.id.to_string());
        ensure_destination_is_absent(&repository_directory).await?;

        let output = self
            .git_client
            .clone_repository(CloneRequest {
                git_url: application.git_url.clone(),
                branch: application.branch.clone(),
                destination: repository_directory.clone(),
            })
            .await
            .map_err(DeploymentError::GitCommand)?;

        persist_command_output(
            &self.database,
            application.id,
            "source",
            "stdout",
            &output.stdout,
        )
        .await?;
        persist_command_output(
            &self.database,
            application.id,
            "source",
            "stderr",
            &output.stderr,
        )
        .await?;

        if !output.success {
            return Err(DeploymentError::GitCloneFailed(output.exit_code));
        }

        let build_context =
            validate_build_context(&repository_directory, &application.build_context).await?;
        repository::append_log(
            &self.database,
            application.id,
            "source",
            "system",
            &format!(
                "source ready; build context validated at {}",
                application.build_context
            ),
        )
        .await?;
        repository::transition_status(
            &self.database,
            application.id,
            ApplicationStatus::Cloning,
            ApplicationStatus::SourceReady,
        )
        .await?;

        tracing::info!(
            application_id = %application.id,
            build_context = %build_context.display(),
            "source preparation completed"
        );

        Ok(build_context)
    }

    async fn build_image(
        &self,
        application: &Application,
        build_context: PathBuf,
    ) -> Result<(), DeploymentError> {
        repository::transition_status(
            &self.database,
            application.id,
            ApplicationStatus::SourceReady,
            ApplicationStatus::Building,
        )
        .await?;

        let image_tag = image_tag(application.id);
        repository::append_log(
            &self.database,
            application.id,
            "build",
            "system",
            &format!("starting Docker image build as {image_tag}"),
        )
        .await?;

        let output = self
            .docker_client
            .build_image(BuildRequest {
                context: build_context,
                image_tag: image_tag.clone(),
                labels: image_labels(application),
            })
            .await
            .map_err(DeploymentError::DockerCommand)?;

        persist_command_output(
            &self.database,
            application.id,
            "build",
            "stdout",
            &output.stdout,
        )
        .await?;
        persist_command_output(
            &self.database,
            application.id,
            "build",
            "stderr",
            &output.stderr,
        )
        .await?;

        if !output.success {
            return Err(DeploymentError::DockerBuildFailed(output.exit_code));
        }

        repository::append_log(
            &self.database,
            application.id,
            "build",
            "system",
            &format!("Docker image ready as {image_tag}"),
        )
        .await?;
        repository::transition_status(
            &self.database,
            application.id,
            ApplicationStatus::Building,
            ApplicationStatus::ImageReady,
        )
        .await?;

        tracing::info!(
            application_id = %application.id,
            %image_tag,
            "Docker image build completed"
        );

        Ok(())
    }
}

pub(crate) fn image_tag(application_id: uuid::Uuid) -> String {
    format!("izyploy/application:{application_id}")
}

fn image_labels(application: &Application) -> Vec<(String, String)> {
    vec![
        ("com.izyploy.managed".to_owned(), "true".to_owned()),
        (
            "com.izyploy.application.id".to_owned(),
            application.id.to_string(),
        ),
        (
            "com.izyploy.application.name".to_owned(),
            application.name.clone(),
        ),
    ]
}

async fn persist_command_output(
    database: &SqlitePool,
    application_id: uuid::Uuid,
    stage: &'static str,
    stream: &'static str,
    output: &str,
) -> Result<(), sqlx::Error> {
    if output.trim().is_empty() {
        return Ok(());
    }

    repository::append_log(database, application_id, stage, stream, output).await
}

async fn ensure_destination_is_absent(destination: &Path) -> Result<(), DeploymentError> {
    match fs::symlink_metadata(destination).await {
        Ok(_) => Err(DeploymentError::WorkspaceAlreadyExists),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(DeploymentError::Workspace(error)),
    }
}

async fn validate_build_context(
    repository_directory: &Path,
    build_context: &str,
) -> Result<PathBuf, DeploymentError> {
    let repository_root = fs::canonicalize(repository_directory).await?;
    let context = fs::canonicalize(repository_root.join(build_context))
        .await
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => DeploymentError::BuildContextMissing,
            _ => DeploymentError::Workspace(error),
        })?;

    if !context.starts_with(&repository_root) {
        return Err(DeploymentError::BuildContextOutsideRepository);
    }

    if !fs::metadata(&context).await?.is_dir() {
        return Err(DeploymentError::BuildContextNotDirectory);
    }

    let dockerfile = context.join("Dockerfile");
    let dockerfile_metadata =
        fs::symlink_metadata(&dockerfile)
            .await
            .map_err(|error| match error.kind() {
                ErrorKind::NotFound => DeploymentError::DockerfileMissing,
                _ => DeploymentError::Workspace(error),
            })?;

    if !dockerfile_metadata.is_file() {
        return Err(DeploymentError::DockerfileNotRegularFile);
    }

    Ok(context)
}

#[derive(Debug)]
enum DeploymentError {
    Database(sqlx::Error),
    Workspace(std::io::Error),
    GitCommand(std::io::Error),
    DockerCommand(std::io::Error),
    WorkspaceAlreadyExists,
    GitCloneFailed(Option<i32>),
    BuildContextMissing,
    BuildContextOutsideRepository,
    BuildContextNotDirectory,
    DockerfileMissing,
    DockerfileNotRegularFile,
    DockerBuildFailed(Option<i32>),
}

impl Display for DeploymentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(_) => formatter.write_str("database operation failed"),
            Self::Workspace(error) => write!(formatter, "workspace operation failed: {error}"),
            Self::GitCommand(error) => write!(formatter, "Git command failed to execute: {error}"),
            Self::DockerCommand(error) => {
                write!(formatter, "Docker command failed to execute: {error}")
            }
            Self::WorkspaceAlreadyExists => {
                formatter.write_str("application workspace already exists")
            }
            Self::GitCloneFailed(Some(exit_code)) => {
                write!(formatter, "Git clone failed with exit code {exit_code}")
            }
            Self::GitCloneFailed(None) => formatter.write_str("Git clone was terminated"),
            Self::BuildContextMissing => formatter.write_str("build context does not exist"),
            Self::BuildContextOutsideRepository => {
                formatter.write_str("build context resolves outside the cloned repository")
            }
            Self::BuildContextNotDirectory => {
                formatter.write_str("build context is not a directory")
            }
            Self::DockerfileMissing => {
                formatter.write_str("Dockerfile is missing from the build context root")
            }
            Self::DockerfileNotRegularFile => {
                formatter.write_str("Dockerfile must be a regular file")
            }
            Self::DockerBuildFailed(Some(exit_code)) => {
                write!(
                    formatter,
                    "Docker image build failed with exit code {exit_code}"
                )
            }
            Self::DockerBuildFailed(None) => {
                formatter.write_str("Docker image build was terminated")
            }
        }
    }
}

impl DeploymentError {
    fn stage(&self) -> &'static str {
        match self {
            Self::DockerCommand(_) | Self::DockerBuildFailed(_) => "build",
            _ => "source",
        }
    }
}

impl std::error::Error for DeploymentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Workspace(error) | Self::GitCommand(error) | Self::DockerCommand(error) => {
                Some(error)
            }
            _ => None,
        }
    }
}

impl From<sqlx::Error> for DeploymentError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<std::io::Error> for DeploymentError {
    fn from(error: std::io::Error) -> Self {
        Self::Workspace(error)
    }
}
