use std::{path::PathBuf, sync::Arc};

use sqlx::SqlitePool;

use crate::{
    applications::source::SourcePreparer,
    git::{CommandGitClient, GitClient},
};

#[derive(Clone)]
pub struct AppState {
    database: SqlitePool,
    source_preparer: Option<SourcePreparer>,
}

impl AppState {
    pub fn new(database: SqlitePool, workspace_root: PathBuf) -> Self {
        Self::with_git_client(database, workspace_root, Arc::new(CommandGitClient))
    }

    pub fn with_git_client(
        database: SqlitePool,
        workspace_root: PathBuf,
        git_client: Arc<dyn GitClient>,
    ) -> Self {
        let source_preparer = SourcePreparer::new(database.clone(), workspace_root, git_client);
        Self {
            database,
            source_preparer: Some(source_preparer),
        }
    }

    pub fn without_source_preparation(database: SqlitePool) -> Self {
        Self {
            database,
            source_preparer: None,
        }
    }

    pub fn database(&self) -> &SqlitePool {
        &self.database
    }

    pub(crate) fn source_preparer(&self) -> Option<&SourcePreparer> {
        self.source_preparer.as_ref()
    }
}
