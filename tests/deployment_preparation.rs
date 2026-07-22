use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode},
};
use izyploy::{
    AppState, app, database,
    docker::{BuildFuture, BuildOutput, BuildRequest, DockerClient},
    git::{CloneFuture, CloneOutput, CloneRequest, GitClient},
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{fs, sync::Notify, time};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn creation_returns_before_deployment_preparation_finishes() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let git_client = FakeGitClient::blocking_success(started.clone(), release.clone());
    let state = test_state(&temporary_directory, git_client).await;

    let response = time::timeout(
        Duration::from_millis(250),
        create_application(state.clone(), "rust"),
    )
    .await
    .expect("HTTP creation should not wait for Git");

    assert_eq!(response.status(), StatusCode::CREATED);
    let queued = response_json(response).await;
    assert_eq!(queued["status"], "queued");
    let application_id = application_id(&queued);

    time::timeout(Duration::from_secs(1), started.notified())
        .await
        .expect("background Git clone should start");
    release.notify_one();

    let ready = wait_for_status(&state, application_id, "image_ready").await;
    assert_eq!(ready["error"], Value::Null);
    assert!(
        temporary_directory
            .path()
            .join("workspaces")
            .join(application_id.to_string())
            .join("rust/Dockerfile")
            .is_file()
    );

    let logs = deployment_logs(&state, application_id).await;
    assert!(
        logs.iter()
            .any(|(_, _, message)| message.contains("starting Git clone"))
    );
    assert!(
        logs.iter()
            .any(|(_, _, message)| message.contains("source ready"))
    );
}

#[tokio::test]
async fn only_one_deployment_preparation_runs_at_a_time() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let git_client = FakeGitClient::blocking_success(started.clone(), release.clone());
    let state = test_state(&temporary_directory, git_client).await;

    let first = response_json(create_application(state.clone(), "rust").await).await;
    let second = response_json(create_application(state.clone(), "rust").await).await;
    let first_id = application_id(&first);
    let second_id = application_id(&second);

    time::timeout(Duration::from_secs(1), started.notified())
        .await
        .expect("one background Git clone should start");

    let statuses = wait_for_serialized_statuses(&state).await;
    let cloning_id = statuses
        .iter()
        .find_map(|(id, status)| (status == "cloning").then_some(*id))
        .expect("one application should be cloning");
    let queued_id = statuses
        .iter()
        .find_map(|(id, status)| (status == "queued").then_some(*id))
        .expect("one application should remain queued");
    assert!([first_id, second_id].contains(&cloning_id));
    assert!([first_id, second_id].contains(&queued_id));
    assert_ne!(cloning_id, queued_id);

    release.notify_one();
    wait_for_status(&state, cloning_id, "image_ready").await;
    release.notify_one();
    wait_for_status(&state, queued_id, "image_ready").await;
}

#[tokio::test]
async fn deployment_permit_remains_held_during_docker_build() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let build_started = Arc::new(Notify::new());
    let release_build = Arc::new(Notify::new());
    let state = test_state_with_docker(
        &temporary_directory,
        FakeGitClient::immediate(FakeOutcome::Success),
        FakeDockerClient::blocking_success(build_started.clone(), release_build.clone()),
    )
    .await;

    let first = response_json(create_application(state.clone(), "rust").await).await;
    let first_id = application_id(&first);
    time::timeout(Duration::from_secs(1), build_started.notified())
        .await
        .expect("first Docker build should start");
    wait_for_status(&state, first_id, "building").await;

    let second = response_json(create_application(state.clone(), "rust").await).await;
    let second_id = application_id(&second);
    let second_while_first_builds = application_by_id(&state, second_id).await;
    assert_eq!(second_while_first_builds["status"], "queued");

    release_build.notify_one();
    wait_for_status(&state, first_id, "image_ready").await;
    time::timeout(Duration::from_secs(1), build_started.notified())
        .await
        .expect("second Docker build should start after the first completes");
    release_build.notify_one();
    wait_for_status(&state, second_id, "image_ready").await;
}

#[tokio::test]
async fn successful_build_uses_managed_tag_labels_and_context() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let docker_client = FakeDockerClient::successful(requests.clone());
    let state = test_state_with_docker(
        &temporary_directory,
        FakeGitClient::immediate(FakeOutcome::Success),
        docker_client,
    )
    .await;

    let created = response_json(create_application(state.clone(), "rust").await).await;
    let application_id = application_id(&created);
    let ready = wait_for_status(&state, application_id, "image_ready").await;

    assert_eq!(ready["error"], Value::Null);
    let request = requests
        .lock()
        .expect("build requests should be readable")
        .first()
        .expect("one Docker build should have been requested")
        .clone();
    assert_eq!(
        request.image_tag,
        format!("izyploy/application:{application_id}")
    );
    assert!(request.context.ends_with(format!("{application_id}/rust")));
    assert!(
        request
            .labels
            .contains(&("com.izyploy.managed".to_owned(), "true".to_owned()))
    );
    assert!(request.labels.contains(&(
        "com.izyploy.application.id".to_owned(),
        application_id.to_string()
    )));

    let logs = deployment_logs(&state, application_id).await;
    assert!(logs.iter().any(|(stage, _, message)| {
        stage == "build" && message.contains("starting Docker image build")
    }));
    assert!(
        logs.iter().any(|(stage, _, message)| {
            stage == "build" && message.contains("Docker image ready")
        })
    );
}

#[tokio::test]
async fn docker_build_failure_transitions_to_failed_and_captures_stderr() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let state = test_state_with_docker(
        &temporary_directory,
        FakeGitClient::immediate(FakeOutcome::Success),
        FakeDockerClient::failing(),
    )
    .await;

    let created = response_json(create_application(state.clone(), "rust").await).await;
    let application_id = application_id(&created);
    let failed = wait_for_status(&state, application_id, "failed").await;

    assert!(
        failed["error"]
            .as_str()
            .is_some_and(|error| error.contains("exit code 1"))
    );
    let logs = deployment_logs(&state, application_id).await;
    assert!(logs.iter().any(|(stage, stream, message)| {
        stage == "build" && stream == "stderr" && message.contains("Dockerfile parse error")
    }));
}

#[tokio::test]
async fn clone_failure_transitions_to_failed_and_captures_stderr() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let state = test_state(
        &temporary_directory,
        FakeGitClient::immediate(FakeOutcome::CloneFailure),
    )
    .await;

    let response = create_application(state.clone(), "rust").await;
    let created = response_json(response).await;
    let application_id = application_id(&created);
    let failed = wait_for_status(&state, application_id, "failed").await;

    assert!(
        failed["error"]
            .as_str()
            .is_some_and(|error| error.contains("exit code 128"))
    );
    let logs = deployment_logs(&state, application_id).await;
    assert!(logs.iter().any(|(_, stream, message)| {
        stream == "stderr" && message.contains("repository not found")
    }));
}

#[tokio::test]
async fn repository_without_dockerfile_transitions_to_failed() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let state = test_state(
        &temporary_directory,
        FakeGitClient::immediate(FakeOutcome::MissingDockerfile),
    )
    .await;

    let response = create_application(state.clone(), "rust").await;
    let created = response_json(response).await;
    let application_id = application_id(&created);
    let failed = wait_for_status(&state, application_id, "failed").await;

    assert!(
        failed["error"]
            .as_str()
            .is_some_and(|error| error.contains("Dockerfile is missing"))
    );
}

#[cfg(unix)]
#[tokio::test]
async fn symlinked_build_context_cannot_escape_repository() {
    let temporary_directory = TempDir::new().expect("temporary directory should be created");
    let state = test_state(
        &temporary_directory,
        FakeGitClient::immediate(FakeOutcome::EscapingContext),
    )
    .await;

    let response = create_application(state.clone(), "escape").await;
    let created = response_json(response).await;
    let application_id = application_id(&created);
    let failed = wait_for_status(&state, application_id, "failed").await;

    assert!(
        failed["error"]
            .as_str()
            .is_some_and(|error| error.contains("outside the cloned repository"))
    );
}

#[derive(Clone)]
struct FakeGitClient {
    outcome: FakeOutcome,
    started: Option<Arc<Notify>>,
    release: Option<Arc<Notify>>,
}

impl FakeGitClient {
    fn immediate(outcome: FakeOutcome) -> Self {
        Self {
            outcome,
            started: None,
            release: None,
        }
    }

    fn blocking_success(started: Arc<Notify>, release: Arc<Notify>) -> Self {
        Self {
            outcome: FakeOutcome::Success,
            started: Some(started),
            release: Some(release),
        }
    }
}

impl GitClient for FakeGitClient {
    fn clone_repository(&self, request: CloneRequest) -> CloneFuture {
        let outcome = self.outcome;
        let started = self.started.clone();
        let release = self.release.clone();

        Box::pin(async move {
            if let Some(started) = started {
                started.notify_one();
            }
            if let Some(release) = release {
                release.notified().await;
            }

            fake_clone(request, outcome).await
        })
    }
}

#[derive(Clone, Copy)]
enum FakeOutcome {
    Success,
    CloneFailure,
    MissingDockerfile,
    EscapingContext,
}

#[derive(Clone)]
struct FakeDockerClient {
    outcome: DockerOutcome,
    requests: Arc<Mutex<Vec<BuildRequest>>>,
    started: Option<Arc<Notify>>,
    release: Option<Arc<Notify>>,
}

impl FakeDockerClient {
    fn successful(requests: Arc<Mutex<Vec<BuildRequest>>>) -> Self {
        Self {
            outcome: DockerOutcome::Success,
            requests,
            started: None,
            release: None,
        }
    }

    fn failing() -> Self {
        Self {
            outcome: DockerOutcome::Failure,
            requests: Arc::new(Mutex::new(Vec::new())),
            started: None,
            release: None,
        }
    }

    fn blocking_success(started: Arc<Notify>, release: Arc<Notify>) -> Self {
        Self {
            outcome: DockerOutcome::Success,
            requests: Arc::new(Mutex::new(Vec::new())),
            started: Some(started),
            release: Some(release),
        }
    }
}

impl DockerClient for FakeDockerClient {
    fn build_image(&self, request: BuildRequest) -> BuildFuture {
        self.requests
            .lock()
            .expect("build requests should be writable")
            .push(request);
        let outcome = self.outcome;
        let started = self.started.clone();
        let release = self.release.clone();

        Box::pin(async move {
            if let Some(started) = started {
                started.notify_one();
            }
            if let Some(release) = release {
                release.notified().await;
            }

            Ok(match outcome {
                DockerOutcome::Success => BuildOutput {
                    success: true,
                    exit_code: Some(0),
                    stdout: "image built".to_owned(),
                    stderr: String::new(),
                },
                DockerOutcome::Failure => BuildOutput {
                    success: false,
                    exit_code: Some(1),
                    stdout: String::new(),
                    stderr: "Dockerfile parse error".to_owned(),
                },
            })
        })
    }
}

#[derive(Clone, Copy)]
enum DockerOutcome {
    Success,
    Failure,
}

async fn fake_clone(request: CloneRequest, outcome: FakeOutcome) -> io::Result<CloneOutput> {
    if matches!(outcome, FakeOutcome::CloneFailure) {
        return Ok(CloneOutput {
            success: false,
            exit_code: Some(128),
            stdout: String::new(),
            stderr: "fatal: repository not found".to_owned(),
        });
    }

    fs::create_dir_all(&request.destination).await?;

    match outcome {
        FakeOutcome::Success => {
            let context = request.destination.join("rust");
            fs::create_dir_all(&context).await?;
            fs::write(context.join("Dockerfile"), "FROM scratch\n").await?;
        }
        FakeOutcome::MissingDockerfile => {
            fs::create_dir_all(request.destination.join("rust")).await?;
        }
        FakeOutcome::EscapingContext => create_escaping_context(&request.destination).await?,
        FakeOutcome::CloneFailure => unreachable!(),
    }

    Ok(CloneOutput {
        success: true,
        exit_code: Some(0),
        stdout: "clone completed".to_owned(),
        stderr: String::new(),
    })
}

#[cfg(unix)]
async fn create_escaping_context(destination: &std::path::Path) -> io::Result<()> {
    use std::os::unix::fs::symlink;

    let outside = destination
        .parent()
        .expect("workspace should have a parent")
        .join("outside-context");
    fs::create_dir_all(&outside).await?;
    fs::write(outside.join("Dockerfile"), "FROM scratch\n").await?;
    symlink(outside, destination.join("escape"))
}

#[cfg(not(unix))]
async fn create_escaping_context(_destination: &std::path::Path) -> io::Result<()> {
    unreachable!()
}

async fn test_state(temporary_directory: &TempDir, git_client: FakeGitClient) -> AppState {
    test_state_with_docker(
        temporary_directory,
        git_client,
        FakeDockerClient::successful(Arc::new(Mutex::new(Vec::new()))),
    )
    .await
}

async fn test_state_with_docker(
    temporary_directory: &TempDir,
    git_client: FakeGitClient,
    docker_client: FakeDockerClient,
) -> AppState {
    let database = database::connect(&format!(
        "sqlite://{}",
        temporary_directory.path().join("izyploy.db").display()
    ))
    .await
    .expect("test database should connect and migrate");

    AppState::with_clients(
        database,
        temporary_directory.path().join("workspaces"),
        Arc::new(git_client),
        Arc::new(docker_client),
    )
}

async fn create_application(state: AppState, build_context: &str) -> Response<Body> {
    send_json(
        app(state),
        Method::POST,
        "/applications",
        json!({
            "name": "hello-rust",
            "git_url": "https://github.com/example/izyploy-examples.git",
            "branch": "main",
            "build_context": build_context,
            "container_port": 8080
        }),
    )
    .await
}

async fn wait_for_status(state: &AppState, id: Uuid, expected_status: &str) -> Value {
    let deadline = time::Instant::now() + Duration::from_secs(2);

    loop {
        let response = send_empty(
            app(state.clone()),
            Method::GET,
            &format!("/applications/{id}"),
        )
        .await;
        let application = response_json(response).await;

        if application["status"] == expected_status {
            return application;
        }
        assert_ne!(
            application["status"], "failed",
            "deployment preparation failed unexpectedly: {application}"
        );
        assert!(
            time::Instant::now() < deadline,
            "application did not reach {expected_status}: {application}"
        );

        time::sleep(Duration::from_millis(10)).await;
    }
}

async fn application_by_id(state: &AppState, id: Uuid) -> Value {
    let response = send_empty(
        app(state.clone()),
        Method::GET,
        &format!("/applications/{id}"),
    )
    .await;
    response_json(response).await
}

async fn deployment_logs(state: &AppState, id: Uuid) -> Vec<(String, String, String)> {
    sqlx::query_as(
        "SELECT stage, stream, message
         FROM deployment_logs
         WHERE application_id = ?
         ORDER BY id ASC",
    )
    .bind(id.to_string())
    .fetch_all(state.database())
    .await
    .expect("deployment logs should be readable")
}

async fn wait_for_serialized_statuses(state: &AppState) -> Vec<(Uuid, String)> {
    let deadline = time::Instant::now() + Duration::from_secs(2);

    loop {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, status
             FROM applications
             ORDER BY created_at ASC, id ASC",
        )
        .fetch_all(state.database())
        .await
        .expect("application statuses should be readable");
        let statuses = rows
            .into_iter()
            .map(|(id, status)| {
                (
                    Uuid::parse_str(&id).expect("stored application id should be a UUID"),
                    status,
                )
            })
            .collect::<Vec<_>>();
        let cloning = statuses
            .iter()
            .filter(|(_, status)| status == "cloning")
            .count();
        let queued = statuses
            .iter()
            .filter(|(_, status)| status == "queued")
            .count();

        if cloning == 1 && queued == 1 {
            return statuses;
        }
        assert!(
            time::Instant::now() < deadline,
            "source preparations were not serialized: {statuses:?}"
        );
        time::sleep(Duration::from_millis(10)).await;
    }
}

fn application_id(application: &Value) -> Uuid {
    Uuid::parse_str(
        application["id"]
            .as_str()
            .expect("application should have an id"),
    )
    .expect("application id should be a UUID")
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
