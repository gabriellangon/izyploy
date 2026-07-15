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
- Status: accepted
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

## Open decisions

The following proposals must be validated at the relevant milestone before becoming accepted decisions:

- Axum as the Rust HTTP framework;
- SQLite with SQLx for MVP persistence;
- Docker CLI for the first integration, with a possible later move to `bollard`;
- Tokio background tasks before introducing a distributed queue;
- dynamic host ports before introducing Traefik.

## Current state

- Current milestone: milestone 1 — manual Docker workflow.
- Current branch: `feat/milestone-1-docker-manual`.
- Application code: not started.
- Next action: select or create a trusted public test repository, then execute the complete Docker workflow manually.

