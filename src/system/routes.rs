use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::AppState;

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}
