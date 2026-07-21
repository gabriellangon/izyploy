use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;

use crate::{AppState, error::ApiError};

use super::{
    model::{Application, CreateApplicationRequest},
    repository,
    validation::validate,
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/applications", post(create).get(list))
        .route("/applications/{id}", get(find_by_id))
}

async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<Application>), ApiError> {
    let new_application = validate(request).map_err(ApiError::validation)?;
    let application = repository::create(state.database(), new_application).await?;

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
    let id = Uuid::parse_str(&id).map_err(|_| ApiError::InvalidApplicationId)?;
    let application = repository::find_by_id(state.database(), id)
        .await?
        .ok_or(ApiError::ApplicationNotFound)?;

    Ok(Json(application))
}
