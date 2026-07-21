use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct AppState {
    database: SqlitePool,
}

impl AppState {
    pub fn new(database: SqlitePool) -> Self {
        Self { database }
    }

    pub fn database(&self) -> &SqlitePool {
        &self.database
    }
}
