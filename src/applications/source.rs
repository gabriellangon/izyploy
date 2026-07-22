use std::{
    fmt::{Display, Formatter},
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use sqlx::SqlitePool;
use tokio::{fs, sync::Semaphore};

use crate::git::{CloneRequest, GitClient};

use super::{
    model::{Application, ApplicationStatus},
    repository,
};

#[derive(Clone)]
pub(crate) struct SourcePreparer {
    database: SqlitePool,
    workspace_root: Arc<PathBuf>,
    git_client: Arc<dyn GitClient>,
    concurrency: Arc<Semaphore>,
}

impl SourcePreparer {
    pub(crate) fn new(
        database: SqlitePool,
        workspace_root: PathBuf,
        git_client: Arc<dyn GitClient>,
    ) -> Self {
        Self {
            database,
            workspace_root: Arc::new(workspace_root),
            git_client,
            concurrency: Arc::new(Semaphore::new(1)),
        }
    }

    pub(crate) async fn prepare(&self, application: Application) {
        let _permit = self
            .concurrency
            .acquire()
            .await
            .expect("source preparation semaphore should remain open");

        if let Err(error) = self.prepare_source(&application).await {
            tracing::error!(application_id = %application.id, %error, "source preparation failed");

            if let Err(log_error) = repository::append_log(
                &self.database,
                application.id,
                "source",
                "system",
                &error.to_string(),
            )
            .await
            {
                tracing::error!(application_id = %application.id, %log_error, "failed to persist source error log");
            }

            if let Err(status_error) =
                repository::mark_failed(&self.database, application.id, &error.to_string()).await
            {
                tracing::error!(application_id = %application.id, %status_error, "failed to persist failed status");
            }
        }
    }

    async fn prepare_source(&self, application: &Application) -> Result<(), SourceError> {
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
            .await?;

        persist_command_output(&self.database, application.id, "stdout", &output.stdout).await?;
        persist_command_output(&self.database, application.id, "stderr", &output.stderr).await?;

        if !output.success {
            return Err(SourceError::GitCloneFailed(output.exit_code));
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

        Ok(())
    }
}

async fn persist_command_output(
    database: &SqlitePool,
    application_id: uuid::Uuid,
    stream: &'static str,
    output: &str,
) -> Result<(), sqlx::Error> {
    if output.trim().is_empty() {
        return Ok(());
    }

    repository::append_log(database, application_id, "source", stream, output).await
}

async fn ensure_destination_is_absent(destination: &Path) -> Result<(), SourceError> {
    match fs::symlink_metadata(destination).await {
        Ok(_) => Err(SourceError::WorkspaceAlreadyExists),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SourceError::Io(error)),
    }
}

async fn validate_build_context(
    repository_directory: &Path,
    build_context: &str,
) -> Result<PathBuf, SourceError> {
    let repository_root = fs::canonicalize(repository_directory).await?;
    let context = fs::canonicalize(repository_root.join(build_context))
        .await
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => SourceError::BuildContextMissing,
            _ => SourceError::Io(error),
        })?;

    if !context.starts_with(&repository_root) {
        return Err(SourceError::BuildContextOutsideRepository);
    }

    if !fs::metadata(&context).await?.is_dir() {
        return Err(SourceError::BuildContextNotDirectory);
    }

    let dockerfile = context.join("Dockerfile");
    let dockerfile_metadata =
        fs::symlink_metadata(&dockerfile)
            .await
            .map_err(|error| match error.kind() {
                ErrorKind::NotFound => SourceError::DockerfileMissing,
                _ => SourceError::Io(error),
            })?;

    if !dockerfile_metadata.is_file() {
        return Err(SourceError::DockerfileNotRegularFile);
    }

    Ok(context)
}

#[derive(Debug)]
enum SourceError {
    Database(sqlx::Error),
    Io(std::io::Error),
    WorkspaceAlreadyExists,
    GitCloneFailed(Option<i32>),
    BuildContextMissing,
    BuildContextOutsideRepository,
    BuildContextNotDirectory,
    DockerfileMissing,
    DockerfileNotRegularFile,
}

impl Display for SourceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(_) => formatter.write_str("database operation failed"),
            Self::Io(error) => write!(formatter, "workspace operation failed: {error}"),
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
        }
    }
}

impl std::error::Error for SourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for SourceError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<std::io::Error> for SourceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
