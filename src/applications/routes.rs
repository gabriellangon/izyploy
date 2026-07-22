use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;

use crate::{AppState, error::ApiError};

use super::{
    model::{Application, CreateApplicationRequest, DeploymentLog},
    repository,
    validation::validate,
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/applications", post(create).get(list))
        .route(
            "/applications/{id}",
            get(find_by_id).delete(delete_application),
        )
        .route("/applications/{id}/logs", get(list_logs))
}

async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<Application>), ApiError> {
    let new_application = validate(request).map_err(ApiError::validation)?;
    let application = repository::create(state.database(), new_application).await?;

    if let Some(deployment_preparer) = state.deployment_preparer().cloned() {
        let queued_application = application.clone();
        tokio::spawn(async move {
            deployment_preparer.prepare(queued_application).await;
        });
    }

    Ok((StatusCode::CREATED, Json(application)))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Application>>, ApiError> {
    let applications = repository::list(state.database()).await?;

    Ok(Json(applications))
}

async fn find_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Application>, ApiError> {
    let id = parse_application_id(&id)?;
    let application = repository::find_by_id(state.database(), id)
        .await?
        .ok_or(ApiError::ApplicationNotFound)?;

    Ok(Json(application))
}

async fn list_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<DeploymentLog>>, ApiError> {
    let id = parse_application_id(&id)?;
    repository::find_by_id(state.database(), id)
        .await?
        .ok_or(ApiError::ApplicationNotFound)?;
    let logs = repository::list_logs(state.database(), id).await?;

    Ok(Json(logs))
}

async fn delete_application(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let id = parse_application_id(&id)?;
    let deployment_preparer = state
        .deployment_preparer()
        .ok_or_else(|| ApiError::internal("deployment lifecycle is unavailable"))?;
    deployment_preparer
        .delete(id)
        .await
        .map_err(ApiError::internal)?;

    Ok(StatusCode::NO_CONTENT)
}

fn parse_application_id(id: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(id).map_err(|_| ApiError::InvalidApplicationId)
}
