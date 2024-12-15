CREATE TABLE IF NOT EXISTS documents (
    vault_id TEXT NOT NULL,
    vault_update_id INTEGER NOT NULL,
    document_id TEXT UNIQUE NOT NULL,
    relative_path TEXT NOT NULL,
    created_date TIMESTAMP NOT NULL,
    updated_date TIMESTAMP NOT NULL,
    content BLOB NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    PRIMARY KEY (vault_id, vault_update_id)
);

CREATE VIEW IF NOT EXISTS latest_document_versions AS
SELECT d.*
FROM documents d
INNER JOIN (
    SELECT vault_id, MAX(vault_update_id) AS max_version_id
    FROM documents
    GROUP BY vault_id, document_id
) max_versions
ON d.vault_id = max_versions.vault_id
AND d.vault_update_id = max_versions.max_version_id;

CREATE INDEX IF NOT EXISTS idx_documents_vault_id_relative_path
ON documents (vault_id, relative_path);

CREATE UNIQUE INDEX IF NOT EXISTS idx_documents_vault_id_document_id
ON documents (vault_id, document_id);
