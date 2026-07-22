use std::path::PathBuf;

use izyploy::{AppState, app, database};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://izyploy.db".to_owned());
    let database = database::connect(&database_url).await?;
    let workspace_root = std::env::var("WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/workspaces"));
    let state = AppState::new(database, workspace_root);

    let listener = TcpListener::bind(("127.0.0.1", 3000)).await?;
    let address = listener.local_addr()?;

    tracing::info!(%address, "Izyploy API listening");

    axum::serve(listener, app(state)).await?;

    Ok(())
}
