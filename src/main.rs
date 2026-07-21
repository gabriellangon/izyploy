use izyploy::{AppState, app};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let listener = TcpListener::bind(("127.0.0.1", 3000)).await?;
    let address = listener.local_addr()?;

    tracing::info!(%address, "Izyploy API listening");

    axum::serve(listener, app(AppState)).await
}
