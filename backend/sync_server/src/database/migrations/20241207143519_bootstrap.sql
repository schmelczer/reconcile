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

CREATE VIEW IF NOT EXISTS latest_documents AS
SELECT d.*
FROM documents d
INNER JOIN (
    SELECT vault_id, document_id, MAX(version_id) AS max_version_id
    FROM documents
    GROUP BY vault_id, document_id
) max_versions
ON d.vault_id = max_versions.vault_id
AND d.document_id = max_versions.document_id
AND d.version_id = max_versions.max_version_id;

CREATE INDEX IF NOT EXISTS idx_documents_vault_doc
ON documents (vault_id, document_id);
