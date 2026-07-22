CREATE TABLE deployment_logs_backup AS
SELECT id, application_id, stage, stream, message, created_at
FROM deployment_logs;

DROP TABLE deployment_logs;

CREATE TABLE applications_with_image_ready (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    git_url TEXT NOT NULL,
    branch TEXT NOT NULL,
    build_context TEXT NOT NULL,
    container_port INTEGER NOT NULL CHECK (container_port BETWEEN 1 AND 65535),
    status TEXT NOT NULL CHECK (
        status IN (
            'queued',
            'cloning',
            'source_ready',
            'building',
            'image_ready',
            'starting',
            'running',
            'failed',
            'deleting'
        )
    ),
    host_port INTEGER CHECK (host_port BETWEEN 1 AND 65535),
    url TEXT,
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

INSERT INTO applications_with_image_ready (
    id,
    name,
    git_url,
    branch,
    build_context,
    container_port,
    status,
    host_port,
    url,
    error,
    created_at,
    updated_at
)
SELECT
    id,
    name,
    git_url,
    branch,
    build_context,
    container_port,
    status,
    host_port,
    url,
    error,
    created_at,
    updated_at
FROM applications;

DROP TABLE applications;
ALTER TABLE applications_with_image_ready RENAME TO applications;

CREATE TABLE deployment_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    stage TEXT NOT NULL,
    stream TEXT NOT NULL CHECK (stream IN ('system', 'stdout', 'stderr')),
    message TEXT NOT NULL,
    created_at TEXT NOT NULL
);

INSERT INTO deployment_logs (id, application_id, stage, stream, message, created_at)
SELECT id, application_id, stage, stream, message, created_at
FROM deployment_logs_backup;

DROP TABLE deployment_logs_backup;

CREATE INDEX deployment_logs_application_id_id
    ON deployment_logs (application_id, id);
