CREATE TABLE applications (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    git_url TEXT NOT NULL,
    branch TEXT NOT NULL,
    build_context TEXT NOT NULL,
    container_port INTEGER NOT NULL CHECK (container_port BETWEEN 1 AND 65535),
    status TEXT NOT NULL CHECK (
        status IN ('queued', 'cloning', 'building', 'starting', 'running', 'failed', 'deleting')
    ),
    host_port INTEGER CHECK (host_port BETWEEN 1 AND 65535),
    url TEXT,
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
