CREATE TABLE IF NOT EXISTS documents (
    vault_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    version_id INTEGER NOT NULL,
    created_date TIMESTAMP NOT NULL,
    updated_date TIMESTAMP NOT NULL,
    relative_path TEXT NOT NULL,
    content BLOB NOT NULL,
    is_binary BOOLEAN NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    PRIMARY KEY (vault_id, document_id, version_id)
);
