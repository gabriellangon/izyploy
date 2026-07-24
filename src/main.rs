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
    let runtime_host = std::env::var("RUNTIME_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let state = AppState::new(database, workspace_root, runtime_host).await?;

    let bind_address =
        std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let listener = TcpListener::bind(&bind_address).await?;
    let address = listener.local_addr()?;

    tracing::info!(%address, "Izyploy API listening");

    axum::serve(listener, app(state)).await?;

    Ok(())
}
