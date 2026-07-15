# Izyploy project instructions

@/Users/gabriel.maomy/.codex/RTK.md

## Required project context

Before changing this repository, read:

1. `spec/product-spec.md`;
2. `spec/implementation-plan.md`;
3. `knowledge.md`.

Treat these files as project context, not optional documentation.

## Learning workflow

- Work on one milestone at a time.
- Explain the relevant concepts before implementing a new milestone.
- Keep changes small, observable, and appropriate to the current milestone.
- Verify each step and summarize what was learned.
- Obtain explicit user validation before moving to the next milestone.
- Do not implement future milestones preemptively.
- Do not complete the project as a large autonomous batch.

## Roadmap tracking

- Treat the `Roadmap status` section at the top of `spec/implementation-plan.md` as the source of truth for progress.
- Update its date, active milestone, checkboxes, and immediate subtasks in the same change that alters project progress.
- Keep at most one milestone marked as in progress.
- Mark a milestone complete only after its validation criteria are satisfied and the user explicitly accepts it.
- Never infer completion only from code being written or committed.

## Project memory

- Update `knowledge.md` whenever work introduces or changes a significant product, architecture, security, workflow, naming, or tooling decision.
- Include the decision, its rationale, its date, and its status.
- Preserve superseded decisions instead of silently rewriting history.
- Do not add routine activity logs to `knowledge.md`.

## Milestone learning documents

- Create a learning document when a milestone introduces application code, infrastructure, or a significant concept that benefits from a reproducible explanation.
- Store these documents under `docs/milestones/`.
- Name them `milestone-<zero-padded-number>-<english-topic>.md`, for example `milestone-02-rust-api-skeleton.md`.
- Summarize the objective, concepts, implementation, verification, known limits, and key learning outcomes when those sections are relevant.
- Reference each document from the corresponding detailed milestone in `spec/implementation-plan.md`.
- Do not duplicate roadmap status or durable architectural decisions in these documents.

## User-facing commands

- Use `rtk` for agent shell execution as required by the imported RTK instructions.
- Never include `rtk` or `rtk proxy` in user-facing documentation, examples, or commands.
- Write documented commands with the standard project CLI, such as `git`, `docker`, `cargo`, or `curl`.
- Treat RTK as internal agent tooling, not as a project dependency or user prerequisite.

## Naming and Git

- Use English for branch names, commit messages, code identifiers, application filenames, APIs, database objects, and infrastructure resources.
- Use lowercase kebab-case branch names with a standard prefix.
- Use short English Conventional Commit messages.
- Use one short-lived branch per ordinary milestone.
- Keep `main` stable and demonstrable.
- Before milestone 15, preserve the Docker version with a tag and a long-lived `docker` branch.
- Never rewrite published history or remove the preserved Docker history during the Kubernetes migration.

## Scope control

- The current source of truth for scope is `spec/product-spec.md`.
- The current source of truth for sequencing is `spec/implementation-plan.md`.
- If implementation requires a decision that is not documented, stop at the decision boundary, discuss it with the user, then record the accepted choice in `knowledge.md`.
