# Milestone 6 — Application runtime and exposure

## Objective

Complete the first vertical Izyploy deployment path by starting the image produced in milestone 5 and exposing the application through a dynamically allocated local port.

```text
queued → cloning → source_ready → building → image_ready → starting → running
```

At `running`, the application record contains both the assigned host port and a usable local URL.

## Core concepts

### Container versus image

An image is an immutable build result. A container is a running instance of that image with its own name, environment, network mapping, resource limits, and logs. Milestone 5 created the image; milestone 6 creates the container.

### Dynamic port publication

The application listens on its declared internal `container_port`. Izyploy passes the same value as `PORT` and asks Docker to select an unused host port. The binding is restricted to `127.0.0.1`, so it is accessible only from the host until public routing is introduced.

### Resource limits

The initial runtime contract applies:

- 1 CPU;
- 512 MiB of memory;
- 256 processes.

These limits reduce accidental host exhaustion but are not a security boundary for hostile code.

### Readiness

A successfully created container is not necessarily ready to receive traffic. Izyploy discovers the assigned host port and attempts a TCP connection for up to 30 seconds. This generic probe verifies reachability without requiring every application to implement a particular health endpoint.

### End-to-end serialization

The same Tokio semaphore permit remains held through clone, build, container creation, port discovery, and readiness. A second deployment remains `queued` until the first becomes `running` or fails.

## Implementation

- `src/docker.rs` now supports structured container run, port discovery, and log commands.
- `src/runtime.rs` provides an injectable TCP readiness probe.
- `src/applications/deployment.rs` orchestrates `image_ready → starting → running` and defines deterministic image and container identifiers.
- `src/applications/repository.rs` atomically stores `running`, `host_port`, and `url` from the expected `starting` state.
- `src/state.rs` assembles the real Docker and readiness clients while tests inject deterministic substitutes.
- `tests/deployment_preparation.rs` covers the complete deployment pipeline and runtime failures.

The container is named:

```text
izyploy-app-<application-id>
```

It receives the same ownership labels as its image plus:

```text
com.izyploy.resource.kind=container
```

## Verification

Automated verification covers:

- runtime request names, image tags, labels, environment, and resource limits;
- persistence of the dynamic host port and URL;
- container-run failure and stderr persistence;
- readiness timeout and startup-log persistence;
- one deployment permit remaining held during readiness;
- all earlier clone and build behavior.

The complete verification commands are:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

A real API deployment of the trusted PHP example was also verified. It reached `running` at a Docker-selected loopback port, `/` returned the Hello World payload, and `/health` returned `{"status":"ok"}`. Docker inspection confirmed 1 CPU, 512 MiB, 256 processes, `PORT=8080`, the `127.0.0.1` port binding, and all ownership labels. The temporary container, image, database, and workspace were removed afterward.

## Known limits

- Readiness checks TCP reachability, not application-specific health semantics.
- Startup logs are captured once; continuous runtime log collection is deferred to milestone 7.
- There is no ongoing health monitor, automatic restart, or reconciliation after an Izyploy restart.
- A container that starts but misses readiness may remain until lifecycle cleanup is implemented.
- The generated URL is local-only; public domains, TLS, and reverse-proxy routing come later.
- Docker access and trusted Dockerfiles remain privileged host capabilities.

## Learning outcomes

- Building an image and running a container are distinct lifecycle stages.
- Letting Docker allocate the host port avoids application-side allocation races.
- A persisted URL should be written only after the runtime is reachable.
- Resource limits reduce operational blast radius but do not safely sandbox arbitrary code.
- A TCP probe is a useful generic MVP readiness check, while richer health contracts can be added later.
