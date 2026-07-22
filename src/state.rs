use std::{path::PathBuf, sync::Arc};

use sqlx::SqlitePool;

use crate::{
    applications::deployment::DeploymentPreparer,
    docker::{CommandDockerClient, DockerClient},
    git::{CommandGitClient, GitClient},
};

#[derive(Clone)]
pub struct AppState {
    database: SqlitePool,
    deployment_preparer: Option<DeploymentPreparer>,
}

impl AppState {
    pub fn new(database: SqlitePool, workspace_root: PathBuf) -> Self {
        Self::with_clients(
            database,
            workspace_root,
            Arc::new(CommandGitClient),
            Arc::new(CommandDockerClient),
        )
    }

    pub fn with_clients(
        database: SqlitePool,
        workspace_root: PathBuf,
        git_client: Arc<dyn GitClient>,
        docker_client: Arc<dyn DockerClient>,
    ) -> Self {
        let deployment_preparer =
            DeploymentPreparer::new(database.clone(), workspace_root, git_client, docker_client);
        Self {
            database,
            deployment_preparer: Some(deployment_preparer),
        }
    }

    pub fn without_deployment_preparation(database: SqlitePool) -> Self {
        Self {
            database,
            deployment_preparer: None,
        }
    }

    pub fn database(&self) -> &SqlitePool {
        &self.database
    }

    pub(crate) fn deployment_preparer(&self) -> Option<&DeploymentPreparer> {
        self.deployment_preparer.as_ref()
    }
}
