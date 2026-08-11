pub const MIGRATIONS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS history (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    request_uuid         TEXT,
    collection_path      TEXT,
    request_name         TEXT,
    method                TEXT NOT NULL,
    url                   TEXT NOT NULL,
    request_headers       TEXT,
    request_body          TEXT,
    auth_type             TEXT,
    status_code           INTEGER,
    response_headers      TEXT,
    response_body         TEXT,
    response_size_bytes   INTEGER,
    error_message         TEXT,
    duration_ms           INTEGER,
    sent_at               TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_history_sent_at ON history(sent_at DESC);
CREATE INDEX IF NOT EXISTS idx_history_url ON history(url);
CREATE INDEX IF NOT EXISTS idx_history_req_uuid ON history(request_uuid);
"#;
