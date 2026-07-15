# Izyploy — Project Knowledge

## Purpose

This file is the project's living memory. It records decisions, their rationale, conventions, constraints, and facts that will be useful in later sessions.

The specifications describe what we plan to build. This file explains why important choices were made and how the project is expected to evolve.

## Maintenance rules

- Read this file before starting work on a milestone.
- Update it in the same change that introduces a significant product, architecture, security, workflow, or tooling decision.
- Record the reason for a decision, not only its result.
- Do not silently delete an obsolete decision. Mark it as superseded and link it to its replacement.
- Keep entries concise and factual.
- Do not use this file as a daily activity log or duplicate the implementation plan.
- Review its changes before every merge into `main`.

## Naming conventions

All technical nomenclature must be written in English:

- branch names;
- commit messages;
- code identifiers and module names;
- filenames created for the application;
- API routes and JSON fields;
- database tables and columns;
- Docker resources and configuration keys.

Learning notes and explanatory documentation may remain in French, but technical names used inside them must follow the English nomenclature.

### Branches

Use lowercase kebab-case with a type prefix:

```text
feat/milestone-1-docker-manual
feat/milestone-2-api
fix/container-cleanup
docs/update-deployment-guide
refactor/runtime-interface
```

Long-lived branches are exceptional. `main` contains the latest stable version. A `docker` branch will preserve the last stable Docker-based version before the Kubernetes migration.

### Commits

Use short English Conventional Commit messages:

```text
docs: define project knowledge rules
feat: add health endpoint
fix: remove orphaned container after failed deployment
refactor: extract deployment runtime interface
```

## Decision log

### D-001 — Product direction

- Date: 2026-07-15
- Status: superseded in part by D-009
- Decision: Izyploy is a learning-oriented mini-PaaS that deploys a trusted public Git repository containing a root `Dockerfile`.
- Reason: the workflow is demonstrable while covering practical Platform Engineering concepts without attempting to reproduce a complete commercial platform.

### D-002 — Incremental learning workflow

- Date: 2026-07-15
- Status: accepted
- Decision: work proceeds one milestone at a time with an explanation, a small implementation, verification, a learning review, and explicit validation before continuing.
- Reason: the project exists primarily to learn and must not be implemented autonomously as one large batch.

### D-003 — Initial execution model

- Date: 2026-07-15
- Status: accepted
- Decision: the MVP will use the host Docker Engine. Once Izyploy is containerized, it will access the host socket and create sibling application containers.
- Reason: Docker-outside-of-Docker is simpler to understand and operate than Docker-in-Docker for this project.
- Constraint: access to the Docker socket is effectively administrative access to the host. The MVP must run only trusted repositories.

### D-004 — Git history and milestone isolation

- Date: 2026-07-15
- Status: accepted
- Decision: ordinary milestones use short-lived branches named `feat/milestone-<number>-<topic>`, merged into `main` after review. Important stable states receive version tags.
- Reason: this preserves a readable, company-style history and makes each learning stage easy to inspect.

### D-005 — Docker-to-Kubernetes transition

- Date: 2026-07-15
- Status: accepted
- Decision: milestone 15 is the main architectural transition. Before it starts, the last Docker version will be preserved with a tag and a long-lived `docker` branch. Kubernetes work will begin on `feat/milestone-15-kubernetes-runtime`.
- Reason: the Kubernetes runtime replaces direct container creation, port publication, and Docker/Traefik integration. The Docker implementation must remain available for comparison and demonstration.

### D-006 — Language of technical nomenclature

- Date: 2026-07-15
- Status: accepted
- Decision: all technical nomenclature and Git metadata use English.
- Reason: this matches common professional conventions and keeps the repository consistent for an international audience.

### D-007 — Roadmap tracking

- Date: 2026-07-15
- Status: accepted
- Decision: the `Roadmap status` section at the top of `spec/implementation-plan.md` is the source of truth for completed, active, and pending work. Only one milestone may be active, and completion requires explicit user validation.
- Reason: the detailed plan explains the work but did not provide an immediate view of progress or remaining work.

### D-008 — Initial MVP technical stack

- Date: 2026-07-15
- Status: accepted
- Decision: the MVP uses Rust with Axum, SQLite with SQLx, Tokio background tasks, the Docker CLI, dynamic host ports, local API execution before containerization, and trusted public GitHub repositories only.
- Reason: these choices keep each learning step observable and minimize infrastructure before the core deployment workflow works.
- Constraints:
  - SQLite is suitable for one host but will be reconsidered before distributed workers, where PostgreSQL is the likely replacement.
  - Tokio tasks are intentionally temporary and require restart reconciliation; a durable queue will replace them in milestone 14.
  - Docker commands must be invoked with structured arguments and never by concatenating user input into a shell command.
  - The Docker socket and third-party `Dockerfile` builds are trusted-code-only capabilities in the MVP.
  - Docker must allocate the dynamic host port to avoid application-side port-selection races.

### D-009 — Monorepo build context

- Date: 2026-07-15
- Status: accepted
- Decision: deployment requests accept an optional `build_context`, with `.` as the default. It may identify the repository root or a relative subdirectory. Izyploy always uses a file named `Dockerfile` at the root of that context; a configurable `dockerfile_path` is outside the MVP.
- Reason: this supports common monorepo layouts and lets one modular example repository contain multiple test applications without adding arbitrary Dockerfile-path complexity.
- Constraints:
  - absolute paths and `..` segments are rejected;
  - the resolved directory must remain inside the cloned repository;
  - symlink-based escapes must be rejected;
  - the context must exist and contain a regular root `Dockerfile`.
- Example repository: `izyploy-examples`, with `java`, `php`, `python`, and `rust` contexts.

## Open decisions

No technical decision is currently open for milestone 1. The first application build context used for the manual Docker workflow remains to be selected.

## Current state

- Completed milestone: milestone 0 — project framing.
- Current milestone: milestone 1 — manual Docker workflow.
- Current branch: `feat/milestone-1-docker-manual`.
- Application code: not started.
- Selected test repository: `izyploy-examples`, organized as one application per build-context subdirectory.
- Example repository status: pull request `gabriellangon/izyploy-examples#2` was validated and merged into its `main` branch as commit `c508a3c6aa683d2a5445859da4104b5ae2bf7360`.
- Next action: clone `izyploy-examples` into a temporary workspace, then select the first application build context together.
