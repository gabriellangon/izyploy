use std::time::Instant;

use axum::{
    Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
};

use crate::{AppState, applications::routes::router as application_routes, routes::health::health};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .merge(application_routes())
        .layer(middleware::from_fn(log_request))
        .with_state(state)
}

async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let started_at = Instant::now();
    let response = next.run(request).await;

    tracing::info!(
        %method,
        %uri,
        status = %response.status(),
        latency_ms = started_at.elapsed().as_millis() as u64,
        "HTTP request completed"
    );

    response
}
