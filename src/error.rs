use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::applications::validation::ValidationError;

pub(crate) enum ApiError {
    Validation(ValidationError),
    InvalidApplicationId,
    ApplicationNotFound,
    Database(sqlx::Error),
    Internal(String),
}

impl ApiError {
    pub(crate) fn validation(error: ValidationError) -> Self {
        Self::Validation(error)
    }

    pub(crate) fn internal(error: impl std::fmt::Display) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, field) = match self {
            Self::Validation(error) => (
                StatusCode::BAD_REQUEST,
                "validation_error",
                error.message,
                Some(error.field),
            ),
            Self::InvalidApplicationId => (
                StatusCode::BAD_REQUEST,
                "invalid_application_id",
                "application id must be a UUID".to_owned(),
                Some("id"),
            ),
            Self::ApplicationNotFound => (
                StatusCode::NOT_FOUND,
                "application_not_found",
                "application was not found".to_owned(),
                None,
            ),
            Self::Database(error) => {
                tracing::error!(%error, "database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "an internal error occurred".to_owned(),
                    None,
                )
            }
            Self::Internal(error) => {
                tracing::error!(%error, "internal operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "an internal error occurred".to_owned(),
                    None,
                )
            }
        };

        (
            status,
            Json(ErrorResponse {
                error: ErrorBody {
                    code,
                    message,
                    field,
                },
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<&'static str>,
}
