use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use izyploy::{AppState, app};
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok() {
    let database = SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("in-memory database URL should be valid");
    let response = app(AppState::new(database))
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("health request should be valid"),
        )
        .await
        .expect("health route should respond");

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("health response body should be readable");
    let payload: Value =
        serde_json::from_slice(&body).expect("health response should contain valid JSON");

    assert_eq!(payload, json!({ "status": "ok" }));
}
