use axum::{Json, extract::State};
use serde::Serialize;

use crate::AppState;

#[derive(Debug, Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
}

pub(crate) async fn health(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
