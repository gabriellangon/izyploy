# Milestone 04 — Background Git clone

## Objective

Start source preparation after application creation without keeping the HTTP request open. The worker must clone one trusted public GitHub repository at a time, validate the selected build context, persist diagnostic logs, and finish in either `source_ready` or `failed`.

## Core concepts

### Detached Tokio task

`POST /applications` still validates and inserts the record synchronously. It then spawns a Tokio task with an owned clone of the application and immediately returns `201 Created` with the original `queued` representation.

```text
HTTP request
    ↓ validate and INSERT queued
    ├── return 201 Created
    └── tokio::spawn
            ↓ wait for semaphore
          cloning
            ↓
     source_ready or failed
```

The task is detached from the request future. Dropping the client connection therefore does not cancel an already spawned clone.

### Serialized work

A Tokio semaphore with one permit enforces the initial MVP limit of one source preparation at a time. The task holding the permit transitions to `cloning`; later tasks wait while their applications remain truthfully in `queued`.

This is concurrency control, not a durable queue. The semaphore and waiting futures disappear when the process stops.

### Structured Git execution

`CommandGitClient` invokes the Git executable directly with separate arguments. No shell parses user input.

Its operation is equivalent to:

```bash
git clone --depth 1 --single-branch --branch <branch> -- <git-url> <workspace>
```

The clone is shallow and limited to the requested branch. `GIT_TERMINAL_PROMPT=0` prevents an unattended API process from waiting for interactive credentials.

The client is represented by a small trait. Production uses the command implementation, while tests inject deterministic fakes that can pause, succeed, or fail without external network access.

### Isolated workspace

`WORKSPACE_ROOT` configures the managed workspace root and defaults to `data/workspaces`. Each clone destination is generated only from the application UUID:

```text
data/workspaces/<application-id>/
```

Application names never become filesystem paths. An existing destination is treated as an error rather than overwritten.

### Build-context confinement

After cloning, source preparation:

1. canonicalizes the repository root;
2. joins and canonicalizes the requested `build_context`;
3. verifies that the resolved directory remains below the repository root;
4. verifies that it is a directory;
5. checks `Dockerfile` with symlink metadata and requires a regular file.

Canonicalization detects a build-context symlink pointing outside the repository. Using symlink metadata for `Dockerfile` also rejects a symlink in place of the required regular file.

### Persistent state and logs

Forward-only migrations add the `source_ready` status and a `deployment_logs` table. Existing application data is copied into the new constrained table instead of editing the already published milestone 3 migration.

Logs contain:

- the application identifier;
- a stage such as `source`;
- a stream: `system`, `stdout`, or `stderr`;
- the message and UTC timestamp.

Git output is recorded before a non-zero exit becomes a `failed` application. Unexpected internal details are logged by the API, while the application receives a concise failure message.

## Implementation map

```text
migrations/202607210002_*       forward migration for source_ready
migrations/202607210003_*       persistent deployment logs
src/git.rs                      injectable and command-based Git clients
src/applications/source.rs      source-preparation workflow and confinement
src/applications/repository.rs  status transitions and log persistence
src/applications/routes.rs      background-task launch after creation
src/state.rs                    Git client, workspace root, and shared worker
tests/source_preparation.rs     async, failure, serialization, and escape tests
```

## Verification

Run all automated checks:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Start Izyploy with explicit local paths:

```bash
DATABASE_URL=sqlite://izyploy.db \
WORKSPACE_ROOT=./data/workspaces \
cargo run
```

Submit the trusted example repository:

```bash
curl -i -X POST http://127.0.0.1:3000/applications \
  -H 'content-type: application/json' \
  -d '{
    "name": "hello-php",
    "git_url": "https://github.com/gabriellangon/izyploy-examples.git",
    "branch": "main",
    "build_context": "php",
    "container_port": 8080
  }'
```

The creation response is `queued`. Poll the returned identifier:

```bash
curl -i http://127.0.0.1:3000/applications/<application-id>
```

The expected terminal state for valid source is `source_ready`. Inspect persisted logs and the workspace:

```bash
sqlite3 -header -column izyploy.db \
  'SELECT application_id, stage, stream, message FROM deployment_logs ORDER BY id;'

ls -la data/workspaces/<application-id>/php
```

The manual verification on 2026-07-21 observed an immediate `201 Created` with `queued`, followed by `source_ready` for the real `izyploy-examples` PHP context. The checked-out branch was `main`, the root `Dockerfile` was present, and clone messages were stored in SQLite. Using the same repository root as the build context produced `failed` with a missing-`Dockerfile` error, while a GitLab URL was rejected with `400 Bad Request` before persistence.

## Known limits

- Tokio tasks and the semaphore are process-local and not restart-safe.
- Applications left in `queued` or `cloning` are not reconciled after restart yet.
- Git output is captured at process exit instead of streamed live.
- Partial and failed workspaces are retained until deletion and cleanup are implemented.
- There is no HTTP route for deployment logs yet.
- Clone timeout, cancellation, retry, and repository-size limits are not implemented.
- Public GitHub repository existence and branch existence are learned only when Git runs.
- This remains a trusted-repository workflow, not a sandbox for hostile source.

## Key learning outcomes

- A detached task decouples request latency from long-running work.
- A semaphore limits concurrency but does not provide durability.
- External commands remain safe when arguments bypass a shell.
- Canonical paths are required to enforce a filesystem boundary.
- Persistent state and logs make asynchronous work observable.
- Injecting the Git boundary makes timing and failure behavior deterministic in tests.
