CREATE TABLE IF NOT EXISTS documents (
    vault_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    version_id INTEGER NOT NULL,
    created_date TIMESTAMP NOT NULL,
    updated_date TIMESTAMP NOT NULL,
    content BLOB NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    PRIMARY KEY (vault_id, relative_path, version_id)
);

CREATE VIEW IF NOT EXISTS latest_document_versions AS
SELECT d.*
FROM documents d
INNER JOIN (
    SELECT vault_id, relative_path, MAX(version_id) AS max_version_id
    FROM documents
    GROUP BY vault_id, relative_path
) max_versions
ON d.vault_id = max_versions.vault_id
AND d.relative_path = max_versions.relative_path
AND d.version_id = max_versions.max_version_id;

CREATE INDEX IF NOT EXISTS idx_documents_vault_doc
ON documents (vault_id, relative_path);
