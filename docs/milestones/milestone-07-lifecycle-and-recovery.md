# Milestone 7 — Lifecycle, cleanup, and recovery

## Objective

Make the first complete Izyploy deployment diagnosable, safely removable, and truthful after the orchestrator restarts.

This milestone adds two lifecycle endpoints:

```text
GET    /applications/{id}/logs
DELETE /applications/{id}
```

## Core concepts

### Idempotence

An operation is idempotent when repeating it produces the same final state. Application deletion returns `204 No Content` when cleanup succeeds and also when the application is already absent. A client can therefore retry a delete request after losing the response.

### Cleanup ordering

Izyploy acquires the same single-deployment permit used by creation, then cleans resources in this order:

```text
status → deleting
container removal
image removal
workspace removal
database record removal
```

Deleting the database record last preserves diagnostic state if an earlier cleanup step fails. The foreign key cascade removes deployment logs only after successful resource cleanup.

### Already-absent resources

Docker resources or workspaces may already be gone because of manual intervention or a previous partial request. `No such container`, `No such image`, and a missing workspace are treated as successful cleanup rather than errors.

### Restart recovery

Tokio tasks and the in-memory semaphore do not survive an Izyploy restart. On production state initialization, transient applications in `queued`, `cloning`, `source_ready`, `building`, `image_ready`, `starting`, or `deleting` become `failed`. A recovery log records the interrupted status.

Automatic resume is intentionally avoided because the current system has no durable job lease or step-level idempotency protocol. `running` applications remain untouched because their Docker containers can outlive the API process.

## Implementation

- `src/applications/routes.rs` exposes logs and deletion.
- `src/applications/model.rs` defines the serialized deployment-log model.
- `src/applications/repository.rs` reads logs, manages deletion state, removes records, and transactionally marks interrupted work as failed.
- `src/applications/deployment.rs` serializes cleanup with deployment work and coordinates Docker and filesystem removal.
- `src/docker.rs` removes deterministic containers and images through structured CLI arguments.
- `src/state.rs` runs restart recovery before production traffic is accepted.
- `tests/deployment_preparation.rs` verifies logs, cleanup, idempotence, failure preservation, and restart recovery.

## API behavior

`GET /applications/{id}/logs` returns an array ordered by increasing log ID. Every item contains:

- `id`;
- `application_id`;
- `stage`;
- `stream`;
- `message`;
- `created_at`.

`DELETE /applications/{id}` returns:

- `204 No Content` after successful cleanup;
- `204 No Content` when the application is already absent;
- `400 Bad Request` for a malformed UUID;
- `500 Internal Server Error` when cleanup fails, while retaining the application as `failed`.

## Verification

Automated verification covers:

- chronological logs from source, build, and runtime stages;
- removal of the deterministic container, image, workspace, application, and cascading logs;
- repeated deletion;
- resources already absent;
- cleanup failure retained as a diagnosable failed application;
- startup recovery of an interrupted queued deployment;
- all previous deployment behavior.

The complete verification commands are:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

A real PHP deployment was created through the API, reached `running`, and exposed ordered source, build, and runtime logs. Two consecutive delete requests both returned `204`. Docker inspection, filesystem inspection, and SQLite inspection confirmed that no managed container, image, workspace, or application record remained. The temporary validation database was then removed.

## Known limits

- The logs endpoint is not paginated and is intended for the small MVP log volume.
- Runtime logs are captured during startup rather than streamed continuously.
- Successful deletion removes historical logs with the application.
- Cleanup waits synchronously for the active deployment permit.
- There is no periodic Docker reconciliation or orphan scan.
- Running records are not checked against actual container existence during restart.
- Interrupted deployments are failed, not resumed automatically.

## Learning outcomes

- Idempotent lifecycle operations are safe to retry.
- Resource cleanup should remove the durable record last.
- A partial cleanup failure needs preserved state and logs rather than silent success.
- Process-local asynchronous work requires an explicit restart policy.
- Durable distributed retry will require stronger job ownership and reconciliation than an in-memory semaphore.
