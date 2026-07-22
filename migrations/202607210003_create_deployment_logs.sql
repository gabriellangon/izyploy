CREATE TABLE deployment_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    application_id TEXT NOT NULL REFERENCES applications(id) ON DELETE CASCADE,
    stage TEXT NOT NULL,
    stream TEXT NOT NULL CHECK (stream IN ('system', 'stdout', 'stderr')),
    message TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX deployment_logs_application_id_id
    ON deployment_logs (application_id, id);
