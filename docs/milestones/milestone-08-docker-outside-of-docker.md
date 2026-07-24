# Milestone 8 — Docker-outside-of-Docker

## Objective

Run Izyploy itself in a container while preserving its ability to build images and create application containers on the host Docker Engine.

```text
Host Docker Engine
├── Izyploy container
└── managed application container
```

The application is a sibling of Izyploy, not a nested container.

## Docker-outside-of-Docker

Izyploy mounts the host Docker socket at `/var/run/docker.sock`. Its Docker CLI sends commands through that socket to the host daemon. Images, networks, ports, and application containers therefore belong to the same host engine that runs Izyploy.

This avoids maintaining a nested Docker daemon and reuses the lifecycle developed in the previous milestones. It also creates a critical security boundary: control of the socket is effectively administrative control of the host.

## Image construction

The multi-stage `Dockerfile` uses:

1. `rust:1.85-bookworm` to compile the locked release binary;
2. `docker:29.1.5-cli` to provide a current Docker CLI and Buildx plugin;
3. `debian:bookworm-slim` as the runtime image.

The runtime installs only the additional tools required by Izyploy:

- Git for source cloning;
- CA certificates for HTTPS repositories;
- curl for the container healthcheck;
- the copied Docker CLI and Buildx plugin.

The verified image size was approximately 113 MB.

## Networking

Two different addresses are required:

- `BIND_ADDRESS=0.0.0.0:3000` allows Compose to reach the API inside its container;
- Compose publishes that API only as `127.0.0.1:3000` on the host;
- `RUNTIME_HOST=host.docker.internal` lets Izyploy probe application ports published on the host;
- returned application URLs continue to use host-facing `127.0.0.1` addresses.

Compose adds `host.docker.internal:host-gateway` for Linux compatibility.

## Persistence

The named `izyploy-data` volume is mounted at `/data`:

```text
/data/izyploy.db
/data/workspaces/<application-id>/
```

Restarting or replacing the Izyploy container preserves SQLite and cloned workspaces. Explicit `docker compose down --volumes` removes this data and is therefore a destructive reset operation.

## Files

- `Dockerfile` builds the production image.
- `.dockerignore` excludes Git metadata, local databases, workspaces, and Rust build output.
- `compose.yaml` configures persistence, socket access, networking, loopback publication, health checking, and restart policy.
- `src/main.rs`, `src/state.rs`, and `src/runtime.rs` separate API binding from runtime readiness addressing.

## Verification

The local Rust checks remain:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Container verification uses:

```bash
docker compose config --quiet
docker compose build
docker compose up --detach
docker compose ps
curl http://127.0.0.1:3000/health
```

The complete PHP example was deployed through the containerized API. Docker inspection confirmed that Izyploy and the PHP application were sibling containers. The application reached `running` and returned its Hello World payload through the Docker-selected host port.

The Izyploy container was restarted while the application remained running. The persisted API record and application URL remained available afterward, confirming both control-plane persistence and sibling-container independence. The application, Izyploy container, named volume, network, and test image were removed after verification.

## Known limits

- Docker-socket access is equivalent to host-administrator access.
- Izyploy currently runs as root inside its container to avoid non-portable socket group mapping.
- The architecture remains limited to one Docker host.
- API and application ports remain loopback-only.
- Compose operates Izyploy itself; application repositories still require one Dockerfile and cannot submit Compose files.
- There is no reverse proxy, public domain, or TLS yet.

## Learning outcomes

- Docker-outside-of-Docker and Docker-in-Docker are different architectures.
- A containerized control plane creates sibling workloads through the host socket.
- Container loopback, host loopback, and host-gateway addressing have distinct meanings.
- Persistent control-plane state must live outside the replaceable container filesystem.
- Docker socket convenience comes with an administrative security boundary.
