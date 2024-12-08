use anyhow::{Context, Result};
use models::{
    DocumentId, DocumentVersionId, DocumentVersionWithoutContent, StoredDocumentVersion, VaultId,
};
use sqlx::types::chrono::Utc;
pub mod models;
use sqlx::{sqlite::SqlitePoolOptions, Executor, Pool, Sqlite};

use crate::config::database_config::DatabaseConfig;

#[derive(Clone, Debug)]
pub struct Database {
    connection_pool: Pool<Sqlite>,
}

pub type Transaction<'a> = sqlx::Transaction<'a, Sqlite>;

impl Database {
    pub async fn try_new(config: &DatabaseConfig) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .test_before_acquire(true)
            .connect(&config.sqlite_url)
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to database with url: {}",
                    &config.sqlite_url
                )
            })?;

        Self::run_migrations(&pool).await?;

        Ok(Self {
            connection_pool: pool,
        })
    }

    async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::migrate!("src/database/migrations")
            .run(pool)
            .await
            .context("Cannot check for pending migrations")
    }

    pub async fn create_transaction(&self) -> Result<Transaction<'_>> {
        self.connection_pool
            .begin()
            .await
            .context("Cannot create transaction")
    }

    pub async fn get_latest_documents(
        &self,
        vault: &VaultId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Vec<DocumentVersionWithoutContent>> {
        let query = sqlx::query_as!(
            DocumentVersionWithoutContent,
            r#"
            select 
                d.vault_id,
                d.document_id as "document_id: uuid::Uuid", 
                d.version_id, 
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                d.relative_path,
                d.is_binary,
                d.is_deleted
            from documents d
            where d.vault_id = ? 
                and d.is_deleted = false 
                and d.version_id = (
                    SELECT MAX(sub.version_id)
                    FROM documents sub
                    WHERE sub.vault_id = d.vault_id
                        AND sub.document_id = d.document_id
                )
            "#,
            vault,
        );

        if let Some(transaction) = transaction {
            query.fetch_all(&mut **transaction).await
        } else {
            query.fetch_all(&self.connection_pool).await
        }
        .context("Cannot fetch latest documents")
    }

    pub async fn get_latest_document_version(
        &self,
        vault: &VaultId,
        document: &DocumentId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Option<StoredDocumentVersion>> {
        let query = sqlx::query_as!(
            StoredDocumentVersion,
            r#"
            select 
                vault_id,
                document_id as "document_id: uuid::Uuid", 
                version_id, 
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                relative_path,
                content,
                is_binary,
                is_deleted
            from documents
            where vault_id = ? and document_id = ?
            ORDER BY version_id DESC
            LIMIT 1
            "#,
            vault,
            document
        );

        if let Some(transaction) = transaction {
            query.fetch_optional(&mut **transaction).await
        } else {
            query.fetch_optional(&self.connection_pool).await
        }
        .context("Cannot fetch latest document version")
    }

    pub async fn get_document_version(
        &self,
        vault: &VaultId,
        document: &DocumentId,
        version: &DocumentVersionId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Option<StoredDocumentVersion>> {
        let query = sqlx::query_as!(
            StoredDocumentVersion,
            r#"
            select 
                vault_id,
                document_id as "document_id: uuid::Uuid", 
                version_id, 
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                relative_path,
                content,
                is_binary,
                is_deleted
            from documents
            where vault_id = ? and document_id = ? and version_id = ?"#,
            vault,
            document,
            version
        );

        if let Some(transaction) = transaction {
            query.fetch_optional(&mut **transaction).await
        } else {
            query.fetch_optional(&self.connection_pool).await
        }
        .context("Cannot fetch document version")
    }

    pub async fn insert_document_version(
        &self,
        version: &StoredDocumentVersion,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<()> {
        let query = sqlx::query!(
            r#"
            insert into documents (vault_id, document_id, created_date, updated_date, relative_path, content, is_deleted)
            values (?, ?, ?, ?, ?, ?, ?)
            "#,
            version.vault_id,
            version.document_id,
            version.created_date,
            version.updated_date,
            version.relative_path,
            version.content,
            version.is_deleted
        );

        if let Some(transaction) = transaction {
            query.execute(&mut **transaction).await
        } else {
            query.execute(&self.connection_pool).await
        }
        .context("Cannot insert document version")?;

        Ok(())
    }
}
