# Milestone 03 — Application persistence

## Objective

Accept, validate, store, and retrieve an Izyploy application without cloning its repository or invoking Docker. A successfully created record starts in `queued` and remains available after the API or database connection restarts.

## Core concepts

### Domain model and input DTO

`Application` is the persisted domain model. It includes the generated identifier, normalized request fields, deployment state, future runtime fields, and timestamps.

`CreateApplicationRequest` is a separate data-transfer object containing only client-supplied fields. Separating the two prevents clients from choosing internal values such as `id`, `status`, `host_port`, `error`, or timestamps.

Creation follows this transformation:

```text
JSON request
    ↓ deserialize
CreateApplicationRequest
    ↓ validate and normalize
NewApplication
    ↓ generate internal fields
Application(status = queued)
    ↓ INSERT
SQLite
```

### SQLite migration

The first file under `migrations/` creates the `applications` table. A migration is a versioned schema change: SQLx records applied migrations and runs only the missing ones when a connection is initialized.

Database constraints complement Rust validation. In particular, ports must remain between 1 and 65535 and deployment status must belong to the documented state set.

### SQLx pool and shared state

`database::connect` opens a SQLx connection pool and runs migrations. The pool is stored in `AppState`, so cloned Axum services share database access rather than opening a connection for every request.

The API reads `DATABASE_URL` at startup. When it is absent, the local default is:

```text
sqlite://izyploy.db
```

### Repository and HTTP handlers

The repository module owns SQL statements and conversion between SQLite rows and the domain model. The route handlers remain focused on HTTP concerns:

- deserialize input;
- invoke validation;
- call the repository;
- select the HTTP status and JSON representation.

Implemented routes:

```text
POST /applications
GET  /applications
GET  /applications/{id}
```

`POST` returns `201 Created`. A missing application returns `404 Not Found`, malformed identifiers and invalid creation fields return `400 Bad Request`, and unexpected database details are logged without being exposed to clients.

### Validation boundary

The creation request currently enforces:

- a visible application name from 1 through 100 characters;
- one HTTPS GitHub repository URL in the form `github.com/<owner>/<repository>`;
- a Git-compatible branch name, with `main` as the default;
- `.` or a normalized relative `build_context` without `..`, absolute paths, or backslashes;
- a `container_port` from 1 through 65535.

This build-context check rejects unsafe syntax before persistence. It cannot prove that a directory exists or detect a symlink escape because the repository has not been cloned yet. Those filesystem checks belong to milestone 4.

## Implementation map

```text
migrations/                         versioned SQLite schema
src/database.rs                     pool creation and migration startup
src/state.rs                        shared SQLx pool
src/app.rs                          composition of system and feature routers
src/system/routes.rs                operational endpoints such as health
src/applications/model.rs           DTO, domain model, and deployment statuses
src/applications/validation.rs      creation input validation
src/applications/repository.rs      INSERT and SELECT operations
src/applications/routes.rs          POST and GET handlers
src/error.rs                        stable JSON API errors
tests/applications.rs               API, validation, and restart persistence tests
```

## Verification

Run formatting, static analysis, compilation, and tests:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Start the API with its default local database:

```bash
cargo run
```

Create an application:

```bash
curl -i -X POST http://127.0.0.1:3000/applications \
  -H 'content-type: application/json' \
  -d '{
    "name": "hello-rust",
    "git_url": "https://github.com/example/izyploy-examples.git",
    "branch": "main",
    "build_context": "rust",
    "container_port": 8080
  }'
```

List and retrieve records:

```bash
curl -i http://127.0.0.1:3000/applications
curl -i http://127.0.0.1:3000/applications/<application-id>
```

Inspect the local schema and stored values:

```bash
sqlite3 izyploy.db '.schema applications'
sqlite3 -header -column izyploy.db \
  'SELECT id, name, branch, build_context, container_port, status FROM applications;'
```

The persistence integration test uses a real temporary SQLite file. It creates an application through HTTP, closes the pool, reconnects through the normal migration path, and retrieves the same JSON record.

The manual verification on 2026-07-21 also observed `201 Created` for creation, `200 OK` for list and lookup, a matching `queued` row through the SQLite CLI, and the same UUID and timestamps after stopping and restarting the API against that file.

## Known limits

- Creation does not yet trigger background work.
- Only Izyploy changes deployment statuses; no status-update API exists.
- Names are not unique; the UUID identifies the managed resource.
- JSON syntax and type extraction errors still use Axum's default rejection representation.
- Pagination, filtering, and sorting options are not implemented.
- SQLite write-concurrency tuning and restart reconciliation belong to later operational milestones.
- Repository existence, branch existence, directory existence, `Dockerfile` presence, and symlink confinement cannot be checked until cloning begins.

## Key learning outcomes

- Transport input and persisted domain state serve different purposes.
- Migrations make database structure reproducible and incremental.
- A shared pool gives concurrent handlers controlled database access.
- Validation belongs before persistence, while database constraints provide a second boundary.
- A file-backed integration test can prove persistence across connection restarts.
