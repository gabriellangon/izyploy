# Milestone 02 — Rust API skeleton

## Objective

Create the smallest healthy Izyploy HTTP API before adding persistence or Docker automation. At this stage, the application must start locally, expose `GET /health`, produce useful logs, and be testable without opening a network port.

## Core concepts

### Cargo package

`Cargo.toml` describes the `izyploy` package, its Rust edition, and its dependencies. `Cargo.lock` records the exact dependency versions selected by Cargo so that another checkout builds the same dependency graph.

The package exposes two compilation targets:

- `src/lib.rs` is a library containing the reusable application router;
- `src/main.rs` is the executable that binds a TCP listener and serves that router.

Keeping the router in the library lets tests call it directly without starting a background process or reserving a host port.

### Tokio runtime

Rust futures do nothing until an asynchronous runtime polls them. `#[tokio::main]` creates that runtime for the executable, while `#[tokio::test]` provides one for the HTTP test. The listener, server, and handlers can then wait for I/O without blocking an operating-system thread per connection.

### Axum router and handler

An Axum `Router` maps an HTTP method and path to a handler. The first route is:

```text
GET /health → health handler → 200 OK + {"status":"ok"}
```

The handler returns `Json<HealthResponse>`. Serde serializes this typed Rust structure into JSON, and Axum supplies the appropriate HTTP response metadata.

### Shared state

`AppState` is attached to the router and extracted by the health handler with Axum's `State` extractor. It is intentionally empty in this milestone. This establishes the dependency path used by future handlers without introducing a database, deployment service, or invented configuration before it is needed.

Axum requires shared state to be clonable because router services may be cloned to serve concurrent requests. Later milestones can place clonable service handles inside this structure.

### Structured logging

`tracing-subscriber` installs the process-wide log formatter. The executable records the bound address, and an Axum middleware records the HTTP method, URI, response status, and request duration. This keeps transport-level logging separate from individual handlers.

### In-process HTTP test

The integration test builds the same router as the executable and sends it a request through Tower's `ServiceExt::oneshot`. It verifies both the `200 OK` status and the exact JSON payload. Because no TCP listener is involved, the test is fast and does not depend on a free port.

## Implementation map

```text
Cargo.toml               package metadata and dependencies
src/main.rs              runtime, TCP listener, and server startup
src/lib.rs               public library boundary
src/app.rs               router composition and request logging middleware
src/state.rs             shared application state
src/system/routes.rs     operational router and health handler
tests/health.rs          in-process HTTP contract test
```

The server listens only on `127.0.0.1:3000`. Binding to the loopback interface makes the learning API local to the machine at this stage.

## Verification

Format and inspect the code:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Compile all targets and run the test suite:

```bash
cargo check --all-targets
cargo test
```

Start the API:

```bash
cargo run
```

From another terminal, call the route:

```bash
curl -i http://127.0.0.1:3000/health
```

Expected response:

```http
HTTP/1.1 200 OK
content-type: application/json

{"status":"ok"}
```

The server terminal should also show one startup log and one request-completion log.

## Known limits

- The listening address is fixed rather than configurable.
- The shared state does not contain services yet.
- There is no graceful-shutdown coordination.
- Only the health route exists.
- There is no application model, validation, database, Git operation, or Docker operation.
- Logging uses the default filter and output format; deployment-specific observability belongs to later milestones.

These limits keep the milestone focused. Persistence and application routes belong to milestone 3, while long-running Git and Docker work begins in later milestones.

## Key learning outcomes

- A Rust package can expose a testable library and a runnable binary together.
- Tokio drives asynchronous network work.
- Axum composes typed handlers into a router.
- Shared state is explicit and clonable.
- Middleware handles behavior shared by every route.
- The HTTP contract can be tested without starting a real server.
