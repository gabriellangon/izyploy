# Izyploy

Izyploy deploys Dockerized web applications from public Git repositories.

Provide a repository URL, a branch, a build context, and the application's internal HTTP port. Izyploy clones the source, builds its `Dockerfile`, starts a managed container, captures deployment logs, and returns an address where the application can be reached.

```text
Git repository + build context
              ↓
          Clone source
              ↓
        Build Docker image
              ↓
       Start managed container
              ↓
     Publish application address
```

## Deployment workflow

A deployment request describes the application to run:

```json
{
  "name": "hello-rust",
  "git_url": "https://github.com/gabriellangon/izyploy-examples.git",
  "branch": "main",
  "build_context": "rust",
  "container_port": 8080
}
```

Izyploy then:

1. creates a deployment record;
2. clones the selected branch into an isolated workspace;
3. resolves and validates the requested build context;
4. builds the `Dockerfile` located at the root of that context;
5. starts a named container with resource limits and a published port;
6. records state transitions, logs, and errors;
7. exposes the running application through an HTTP address;
8. removes the container, image, and temporary files when the application is deleted.

The deployment lifecycle uses explicit states:

```text
queued → cloning → building → starting → running
   └────────────── failure ──────────────→ failed
running → deleting
```

## Repository and build contexts

The first version accepts trusted public GitHub repositories containing a `Dockerfile`.

`build_context` may select the repository root or a relative subdirectory. This allows a monorepo to contain several independently deployable applications:

```text
repository/
├── java/
│   └── Dockerfile
├── php/
│   └── Dockerfile
├── python/
│   └── Dockerfile
└── rust/
    └── Dockerfile
```

For example, `build_context: "php"` builds the `php/Dockerfile` with `php/` as the Docker build context. When the field is omitted, Izyploy uses the repository root.

Example applications are maintained in [`gabriellangon/izyploy-examples`](https://github.com/gabriellangon/izyploy-examples).

## Architecture

The initial runtime targets a single Docker host:

```text
HTTP client
    ↓
Izyploy API (Rust + Axum)
    ↓
Deployment orchestrator
    ├── Git client
    ├── Docker client
    └── state and log storage
              ↓
       Host Docker Engine
        ├── application A
        ├── application B
        └── application C
```

The API and orchestrator manage the desired application lifecycle. Docker builds images and runs application containers. SQLite stores the initial application state, while background Tokio tasks execute deployment work.

Later runtime stages add a reverse proxy, public subdomains, HTTPS, redeployment, observability, distributed workers, an image registry, and a Kubernetes execution backend.

## Planned API

```text
GET    /health
POST   /applications
GET    /applications
GET    /applications/{id}
GET    /applications/{id}/logs
DELETE /applications/{id}
```

## Current status

The product specification and implementation roadmap are established. The complete Docker lifecycle has been verified manually with the PHP example application: image build, container start, port publication, HTTP checks, log and metadata inspection, and resource cleanup.

Milestone 5 is complete. One serialized Tokio background pipeline clones and validates trusted GitHub sources, builds labeled Docker images, persists Git and Docker output, and moves successful applications through `source_ready` and `building` to `image_ready`. Milestone 6 is now extending that pipeline through container startup and local HTTP exposure.

## Security boundary

Izyploy's initial Docker runtime is restricted to repositories controlled and trusted by the operator.

Building a third-party `Dockerfile` and controlling the host Docker Engine are privileged operations. Docker alone is not a sufficient isolation boundary for hostile code, and the initial version must not be exposed as a public arbitrary-code execution service.

## Documentation

- [Product specification](spec/product-spec.md)
- [Implementation roadmap](spec/implementation-plan.md)
- [Project decisions](knowledge.md)
- [Milestone 01 — Manual Docker workflow](docs/milestones/milestone-01-manual-docker-workflow.md)
- [Milestone 02 — Rust API skeleton](docs/milestones/milestone-02-rust-api-skeleton.md)
- [Milestone 03 — Application persistence](docs/milestones/milestone-03-application-persistence.md)
- [Milestone 04 — Background Git clone](docs/milestones/milestone-04-background-git-clone.md)
