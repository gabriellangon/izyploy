# Milestone 5 — Docker image build

## Objective

Turn a cloned and validated application source into a Docker image managed by Izyploy, without blocking the HTTP request and without starting a container yet.

The deployment boundary introduced by this milestone is:

```text
queued → cloning → source_ready → building → image_ready
```

Failures at either the source or build stage transition the application to `failed` and persist a diagnostic message.

## Core concepts

### Build context

Docker receives the canonical directory validated during source preparation. Its root `Dockerfile` and all files under that directory form the build context. Izyploy never constructs this path from an unchecked shell command.

### Image tag

Every application receives a deterministic internal tag:

```text
izyploy/application:<application-id>
```

The UUID is already generated and validated by Izyploy, so the tag is safe to pass directly to Docker. Because the tag is reproducible, milestone 6 can identify the image from the application ID without adding another database field.

### Image labels

The image also carries ownership metadata:

```text
com.izyploy.managed=true
com.izyploy.application.id=<application-id>
com.izyploy.application.name=<application-name>
```

Labels allow operators and future cleanup logic to distinguish Izyploy resources from unrelated Docker images.

### Structured process execution

The Docker CLI is launched as a child process with one structured argument per option and value. No shell parses the repository path, image tag, application name, or label values.

### End-to-end permit

The same one-permit Tokio semaphore covers cloning, source validation, and image construction. A second application stays `queued` while the first is cloning or building. This is a deliberately simple in-process queue for the single-host MVP.

## Implementation

- `src/docker.rs` defines the injectable Docker client and the production CLI implementation.
- `src/applications/deployment.rs` owns the complete background preparation pipeline.
- `src/state.rs` assembles the production Git and Docker clients and supports deterministic test doubles.
- `src/applications/model.rs` defines the `image_ready` state.
- `migrations/202607220001_add_image_ready_status.sql` updates the constrained SQLite status values while preserving existing deployment logs.
- `tests/deployment_preparation.rs` covers asynchronous creation, serialization, safe build metadata, success, and failures.

Docker output is stored in `deployment_logs` with stage `build` and stream `stdout` or `stderr`. System messages record the beginning and successful completion of the build.

## Verification

Automated verification covers:

- the HTTP response returning before background work completes;
- one deployment permit spanning the Docker build;
- transition to `image_ready` after a successful build;
- the deterministic tag, canonical build context, and ownership labels;
- transition to `failed` after an intentional Docker failure;
- persistence of Docker stderr and the failure reason.

The complete Rust verification commands are:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

A real API run was also verified with the trusted `izyploy-examples` repository and its `php` context. The application reached `image_ready`; `docker image inspect` confirmed the generated tag and all three Izyploy labels. The temporary image and workspace were removed after inspection.

## Known limits

- Git and Docker command output is persisted only when each command exits, not streamed live.
- The queue and its active task exist only inside one API process and do not resume automatically after a restart.
- Failed builds can retain Docker cache or intermediate resources until lifecycle cleanup is implemented.
- The API does not expose deployment logs yet; this is planned for milestone 7.
- The image is local to the host Docker engine; there is no registry in this milestone.
- Dockerfiles are restricted to repositories trusted by the operator. Building an untrusted Dockerfile can execute arbitrary build instructions against a highly privileged Docker environment.

## Learning outcomes

- A Docker image and a running container are separate lifecycle resources.
- A truthful intermediate state makes milestone boundaries observable and testable.
- Resource tags identify an image conveniently, while labels provide durable ownership metadata.
- An injectable process boundary allows orchestration tests to remain fast and deterministic.
- A semaphore is sufficient for initial serialization, but it is not a durable or distributed job queue.
