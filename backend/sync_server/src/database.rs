use std::str::FromStr;

use anyhow::{Context, Result};
use models::{
    DocumentId, DocumentVersionWithoutContent, StoredDocumentVersion, VaultId, VaultUpdateId,
};
use sqlx::{sqlite::SqliteConnectOptions, types::chrono::Utc};
pub mod models;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use crate::config::database_config::DatabaseConfig;

#[derive(Clone, Debug)]
pub struct Database {
    connection_pool: Pool<Sqlite>,
}

pub type Transaction<'a> = sqlx::Transaction<'a, Sqlite>;

impl Database {
    pub async fn try_new(config: &DatabaseConfig) -> Result<Self> {
        let connection_options = SqliteConnectOptions::from_str(&config.sqlite_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .test_before_acquire(true)
            .connect_with(connection_options)
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

    /// Return the latest state of all non-deleted documents in the vault
    pub async fn get_latest_documents(
        &self,
        vault: &VaultId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Vec<DocumentVersionWithoutContent>> {
        let query = sqlx::query_as!(
            DocumentVersionWithoutContent,
            r#"
            select 
                vault_id,
                vault_update_id,
                document_id as "document_id: uuid::Uuid", 
                relative_path,
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                is_deleted
            from latest_document_versions
            where is_deleted = false and vault_id = ?
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

    /// Return the latest state of all documents (including deleted) in the
    /// vault which have changed since the given update id
    pub async fn get_latest_documents_since(
        &self,
        vault: &VaultId,
        vault_update_id: VaultUpdateId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Vec<DocumentVersionWithoutContent>> {
        let query = sqlx::query_as!(
            DocumentVersionWithoutContent,
            r#"
            select
                vault_id,
                vault_update_id,
                document_id as "document_id: uuid::Uuid",
                relative_path,
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                is_deleted
            from latest_document_versions
            where vault_id = ? and vault_update_id > ?
            "#,
            vault,
            vault_update_id
        );

        if let Some(transaction) = transaction {
            query.fetch_all(&mut **transaction).await
        } else {
            query.fetch_all(&self.connection_pool).await
        }
        .with_context(|| {
            format!("Cannot fetch latest documents since vault_update_id {vault_update_id}")
        })
    }

    pub async fn get_max_update_id_in_vault(
        &self,
        vault: &VaultId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<i64> {
        let query = sqlx::query!(
            r#"
            select coalesce(max(vault_update_id), 0) as max_vault_update_id
            from documents
            where vault_id = ?
            "#,
            vault
        );

        if let Some(transaction) = transaction {
            query.fetch_one(&mut **transaction).await
        } else {
            query.fetch_one(&self.connection_pool).await
        }
        .map(|row| row.max_vault_update_id)
        .context("Cannot fetch max update id in vault")
    }

    pub async fn get_latest_document_by_path(
        &self,
        vault: &VaultId,
        relative_path: &str,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Option<StoredDocumentVersion>> {
        let query = sqlx::query_as!(
            StoredDocumentVersion,
            r#"
            select 
                vault_id,
                vault_update_id,
                document_id as "document_id: uuid::Uuid", 
                relative_path,
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                content,
                is_deleted
            from latest_document_versions
            where vault_id = ? and relative_path = ?
            "#,
            vault,
            relative_path
        );

        if let Some(transaction) = transaction {
            query.fetch_optional(&mut **transaction).await
        } else {
            query.fetch_optional(&self.connection_pool).await
        }
        .context("Cannot fetch latest document version")
    }

    pub async fn get_latest_document(
        &self,
        vault: &VaultId,
        document_id: &DocumentId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Option<StoredDocumentVersion>> {
        let query = sqlx::query_as!(
            StoredDocumentVersion,
            r#"
            select 
                vault_id,
                vault_update_id,
                document_id as "document_id: uuid::Uuid", 
                relative_path,
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                content,
                is_deleted
            from latest_document_versions
            where vault_id = ? and document_id = ?
            "#,
            vault,
            document_id
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
        vault_update_id: VaultUpdateId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Option<StoredDocumentVersion>> {
        let query = sqlx::query_as!(
            StoredDocumentVersion,
            r#"
            select 
                vault_id,
                vault_update_id,
                document_id as "document_id: uuid::Uuid", 
                relative_path,
                created_date as "created_date: chrono::DateTime<Utc>",
                updated_date as "updated_date: chrono::DateTime<Utc>",
                content,
                is_deleted
            from documents
            where vault_id = ? and vault_update_id = ?"#,
            vault,
            vault_update_id
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
            insert into documents (
                vault_id,
                vault_update_id,
                document_id, 
                relative_path,
                created_date,
                updated_date,
                content,
                is_deleted
            )
            values (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            version.vault_id,
            version.vault_update_id,
            version.document_id,
            version.relative_path,
            version.created_date,
            version.updated_date,
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
