use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode},
};
use izyploy::{AppState, app, database};
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn application_survives_database_reconnection() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let database_url = database_url(&temporary_directory);
    let state = test_state(&database_url).await;

    let create_response = send_json(
        app(state.clone()),
        Method::POST,
        "/applications",
        json!({
            "name": "hello-rust",
            "git_url": "https://github.com/example/izyploy-examples.git",
            "container_port": 8080
        }),
    )
    .await;

    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = response_json(create_response).await;
    let application_id = created["id"]
        .as_str()
        .expect("created application should have an id");
    Uuid::parse_str(application_id).expect("application id should be a UUID");
    assert_eq!(created["name"], "hello-rust");
    assert_eq!(created["branch"], "main");
    assert_eq!(created["build_context"], ".");
    assert_eq!(created["container_port"], 8080);
    assert_eq!(created["status"], "queued");
    assert_eq!(created["host_port"], Value::Null);
    assert_eq!(created["url"], Value::Null);
    assert_eq!(created["error"], Value::Null);

    let list_response = send_empty(app(state.clone()), Method::GET, "/applications").await;
    assert_eq!(list_response.status(), StatusCode::OK);
    let applications = response_json(list_response).await;
    assert_eq!(applications.as_array().map(Vec::len), Some(1));
    assert_eq!(applications[0]["id"], application_id);

    state.database().close().await;

    let reconnected_state = test_state(&database_url).await;
    let get_response = send_empty(
        app(reconnected_state),
        Method::GET,
        &format!("/applications/{application_id}"),
    )
    .await;

    assert_eq!(get_response.status(), StatusCode::OK);
    let persisted = response_json(get_response).await;
    assert_eq!(persisted, created);
}

#[tokio::test]
async fn invalid_creation_inputs_are_rejected() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let state = test_state(&database_url(&temporary_directory)).await;
    let cases = [
        (
            json!({
                "name": "hello-rust",
                "git_url": "https://gitlab.com/example/project.git",
                "branch": "main",
                "build_context": ".",
                "container_port": 8080
            }),
            "git_url",
        ),
        (
            json!({
                "name": "hello-rust",
                "git_url": "https://github.com/example/project.git",
                "branch": "feature branch",
                "build_context": ".",
                "container_port": 8080
            }),
            "branch",
        ),
        (
            json!({
                "name": "hello-rust",
                "git_url": "https://github.com/example/project.git",
                "branch": "main",
                "build_context": "../private",
                "container_port": 8080
            }),
            "build_context",
        ),
        (
            json!({
                "name": "hello-rust",
                "git_url": "https://github.com/example/project.git",
                "branch": "main",
                "build_context": ".",
                "container_port": 0
            }),
            "container_port",
        ),
    ];

    for (payload, expected_field) in cases {
        let response = send_json(app(state.clone()), Method::POST, "/applications", payload).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error = response_json(response).await;
        assert_eq!(error["error"]["code"], "validation_error");
        assert_eq!(error["error"]["field"], expected_field);
    }

    let list_response = send_empty(app(state), Method::GET, "/applications").await;
    let applications = response_json(list_response).await;
    assert_eq!(applications, json!([]));
}

#[tokio::test]
async fn missing_and_malformed_application_ids_return_clear_errors() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let state = test_state(&database_url(&temporary_directory)).await;

    let malformed = send_empty(app(state.clone()), Method::GET, "/applications/not-a-uuid").await;
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response_json(malformed).await["error"]["code"],
        "invalid_application_id"
    );

    let missing = send_empty(
        app(state),
        Method::GET,
        &format!("/applications/{}", Uuid::new_v4()),
    )
    .await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response_json(missing).await["error"]["code"],
        "application_not_found"
    );
}

async fn test_state(database_url: &str) -> AppState {
    let database = database::connect(database_url)
        .await
        .expect("test database should connect and migrate");
    AppState::without_source_preparation(database)
}

fn database_url(temporary_directory: &TempDir) -> String {
    format!(
        "sqlite://{}",
        temporary_directory.path().join("izyploy.db").display()
    )
}

async fn send_json(
    application: Router,
    method: Method,
    uri: &str,
    payload: Value,
) -> Response<Body> {
    application
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .expect("JSON request should be valid"),
        )
        .await
        .expect("application should respond")
}

async fn send_empty(application: Router, method: Method, uri: &str) -> Response<Body> {
    application
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .expect("empty request should be valid"),
        )
        .await
        .expect("application should respond")
}

async fn response_json(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&body).expect("response should contain valid JSON")
}
