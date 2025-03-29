use core::time::Duration;
use std::{collections::HashMap, sync::Arc};

use anyhow::{Context as _, Result};
use models::{
    DocumentId, DocumentVersionWithoutContent, StoredDocumentVersion, VaultId, VaultUpdateId,
};
use sqlx::{sqlite::SqliteConnectOptions, types::chrono::Utc};
pub mod models;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use tokio::sync::Mutex;
use uuid::fmt::Hyphenated;

use crate::config::database_config::DatabaseConfig;

#[derive(Clone, Debug)]
pub struct Database {
    config: DatabaseConfig,
    connection_pools: Arc<Mutex<HashMap<VaultId, Pool<Sqlite>>>>,
}

pub type Transaction<'a> = sqlx::Transaction<'a, Sqlite>;

impl Database {
    pub async fn try_new(config: &DatabaseConfig) -> Result<Self> {
        tokio::fs::create_dir_all(&config.databases_directory_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to create databases directory: {}",
                    config.databases_directory_path.to_string_lossy()
                )
            })?;

        let mut connection_pools = std::collections::HashMap::new();

        let mut entries = tokio::fs::read_dir(&config.databases_directory_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_name().to_string_lossy().ends_with(".sqlite") {
                continue;
            }

            let vault: VaultId = entry
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".sqlite")
                .to_owned();

            connection_pools.insert(
                vault.clone(),
                Self::create_vault_database(config, &vault).await?,
            );
        }

        Ok(Self {
            config: config.clone(),
            connection_pools: Arc::new(Mutex::new(connection_pools)),
        })
    }

    async fn create_vault_database(
        config: &DatabaseConfig,
        vault: &VaultId,
    ) -> Result<Pool<Sqlite>> {
        let file_name = config
            .databases_directory_path
            .join(format!("{vault}.sqlite"));

        let connection_options = SqliteConnectOptions::new()
            .filename(file_name.clone())
            .create_if_missing(true)
            .busy_timeout(Duration::from_secs(3600))
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections_per_vault)
            .test_before_acquire(true)
            .connect_with(connection_options)
            .await
            .with_context(|| format!("Cannot open database at {}", file_name.display()))?;

        Self::run_migrations(&pool).await?;

        Ok(pool)
    }

    async fn run_migrations(pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::migrate!("src/app_state/database/migrations")
            .run(pool)
            .await
            .context("Cannot check for pending migrations")
    }

    async fn get_connection_pool(&self, vault: &VaultId) -> Result<Pool<Sqlite>> {
        let mut pools = self.connection_pools.lock().await;
        if !pools.contains_key(vault) {
            let pool = Self::create_vault_database(&self.config, vault).await?;
            pools.insert(vault.clone(), pool);
        }

        let pool = pools
            .get(vault)
            .expect("Pool was just inserted or already exists");

        Ok(pool.clone())
    }

    /// Attempting to write from this transaction might result in a
    /// database locked error. Use this transaction for read-only operations.
    pub async fn create_readonly_transaction(
        &self,
        vault: &VaultId,
    ) -> Result<Transaction<'static>> {
        self.get_connection_pool(vault)
            .await?
            .begin()
            .await
            .context("Cannot create transaction")
    }

    pub async fn create_write_transaction(&self, vault: &VaultId) -> Result<Transaction<'static>> {
        let mut transaction = self.create_readonly_transaction(vault).await?;

        // sqlx doesn't support immediate transactions for sqlite: https://github.com/launchbadge/sqlx/issues/481
        sqlx::query!("END; BEGIN IMMEDIATE;")
            .execute(&mut *transaction)
            .await?;

        Ok(transaction)
    }

    /// Return the latest state of all documents in the vault
    pub async fn get_latest_documents(
        &self,
        vault: &VaultId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Vec<DocumentVersionWithoutContent>> {
        let query = sqlx::query_as!(
            DocumentVersionWithoutContent,
            r#"
            select 
                vault_update_id,
                document_id as "document_id: Hyphenated", 
                relative_path,
                updated_date as "updated_date: chrono::DateTime<Utc>",
                is_deleted
            from latest_document_versions
            order by vault_update_id desc
            "#,
        );

        if let Some(transaction) = transaction {
            query.fetch_all(&mut **transaction).await
        } else {
            query
                .fetch_all(&self.get_connection_pool(vault).await?)
                .await
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
                vault_update_id,
                document_id as "document_id: Hyphenated",
                relative_path,
                updated_date as "updated_date: chrono::DateTime<Utc>",
                is_deleted
            from latest_document_versions
            where vault_update_id > ?
            order by vault_update_id desc
            "#,
            vault_update_id
        );

        if let Some(transaction) = transaction {
            query.fetch_all(&mut **transaction).await
        } else {
            query
                .fetch_all(&self.get_connection_pool(vault).await?)
                .await
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
            "#,
        );

        if let Some(transaction) = transaction {
            query.fetch_one(&mut **transaction).await
        } else {
            query
                .fetch_one(&self.get_connection_pool(vault).await?)
                .await
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
                vault_update_id,
                document_id as "document_id: Hyphenated", 
                relative_path,
                updated_date as "updated_date: chrono::DateTime<Utc>",
                content,
                is_deleted
            from latest_document_versions
            where relative_path = ?
            order by vault_update_id desc  -- `latest_document_versions` only contains a single latest version of each document, however,
                                           -- multiple documents can have the same `relative_path`, if they have been deleted. That's
                                           -- why we only care about the latest version of the document with the given relative path.
            limit 1
            "#,
            relative_path
        );

        if let Some(transaction) = transaction {
            query.fetch_optional(&mut **transaction).await
        } else {
            query
                .fetch_optional(&self.get_connection_pool(vault).await?)
                .await
        }
        .context("Cannot fetch latest document version")
    }

    pub async fn get_latest_document(
        &self,
        vault: &VaultId,
        document_id: &DocumentId,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<Option<StoredDocumentVersion>> {
        let document_id = document_id.as_hyphenated();
        let query = sqlx::query_as!(
            StoredDocumentVersion,
            r#"
            select 
                vault_update_id,
                document_id as "document_id: Hyphenated", 
                relative_path,
                updated_date as "updated_date: chrono::DateTime<Utc>",
                content,
                is_deleted
            from latest_document_versions
            where document_id = ?
            "#,
            document_id
        );

        if let Some(transaction) = transaction {
            query.fetch_optional(&mut **transaction).await
        } else {
            query
                .fetch_optional(&self.get_connection_pool(vault).await?)
                .await
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
                vault_update_id,
                document_id as "document_id: Hyphenated", 
                relative_path,
                updated_date as "updated_date: chrono::DateTime<Utc>",
                content,
                is_deleted
            from documents
            where vault_update_id = ?"#,
            vault_update_id
        );

        if let Some(transaction) = transaction {
            query.fetch_optional(&mut **transaction).await
        } else {
            query
                .fetch_optional(&self.get_connection_pool(vault).await?)
                .await
        }
        .context("Cannot fetch document version")
    }

    pub async fn insert_document_version(
        &self,
        vault: &VaultId,
        version: &StoredDocumentVersion,
        transaction: Option<&mut Transaction<'_>>,
    ) -> Result<()> {
        let document_id = version.document_id.as_hyphenated();
        let query = sqlx::query!(
            r#"
            insert into documents (
                vault_update_id,
                document_id, 
                relative_path,
                updated_date,
                content,
                is_deleted
            )
            values (?, ?, ?, ?, ?, ?)
            "#,
            version.vault_update_id,
            document_id,
            version.relative_path,
            version.updated_date,
            version.content,
            version.is_deleted
        );

        if let Some(transaction) = transaction {
            query.execute(&mut **transaction).await
        } else {
            query.execute(&self.get_connection_pool(vault).await?).await
        }
        .context("Cannot insert document version")?;

        Ok(())
    }
}
