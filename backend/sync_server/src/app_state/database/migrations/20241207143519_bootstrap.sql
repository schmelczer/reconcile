CREATE TABLE IF NOT EXISTS documents (
    vault_update_id INTEGER NOT NULL PRIMARY KEY,
    document_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    updated_date TIMESTAMP NOT NULL,
    content BLOB NOT NULL,
    is_deleted BOOLEAN NOT NULL
);

CREATE VIEW IF NOT EXISTS latest_document_versions AS
SELECT d.*
FROM documents d
INNER JOIN (
    SELECT MAX(vault_update_id) AS max_version_id
    FROM documents
    GROUP BY document_id
) max_versions
ON d.vault_update_id = max_versions.max_version_id;

CREATE INDEX IF NOT EXISTS idx_documents_vault_id_relative_path
ON documents (relative_path);
