use crate::utils::macros::*;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub mod mysql;
pub mod redis;
pub mod sqlite;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn init(&self) -> Result<()>;
    async fn shutdown(&self);

    async fn get(&self, table: &str, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, table: &str, key: &str, value: &[u8]) -> Result<()>;
    async fn delete(&self, table: &str, key: &str) -> Result<()>;
    async fn exists(&self, table: &str, key: &str) -> Result<bool>;
    async fn keys(&self, table: &str, pattern: &str) -> Result<Vec<String>>;

    async fn query(
        &self,
        sql: &str,
        params: &[&dyn sqlx::Encode<'_, sqlx::Any>],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>>;
    async fn execute(&self, sql: &str, params: &[&dyn sqlx::Encode<'_, sqlx::Any>]) -> Result<u64>;
}

pub enum StorageBackend {
    Sqlite(sqlite::SqliteStorage),
    Redis(redis::RedisStorage),
    Mysql(mysql::MysqlStorage),
}

impl StorageBackend {
    pub async fn new(config: &crate::config::StorageConfig) -> Result<Self> {
        match config.backend.as_str() {
            "sqlite" => Ok(StorageBackend::Sqlite(
                sqlite::SqliteStorage::new(&config.sqlite).await?,
            )),
            "redis" => Ok(StorageBackend::Redis(
                redis::RedisStorage::new(&config.redis).await?,
            )),
            "mysql" | "postgres" => Ok(StorageBackend::Mysql(
                mysql::MysqlStorage::new(&config.mysql).await?,
            )),
            _ => Err(anyhow::anyhow!(
                "Unknown storage backend: {}",
                config.backend
            )),
        }
    }
}

#[async_trait]
impl Storage for StorageBackend {
    async fn init(&self) -> Result<()> {
        match self {
            StorageBackend::Sqlite(s) => s.init().await,
            StorageBackend::Redis(s) => s.init().await,
            StorageBackend::Mysql(s) => s.init().await,
        }
    }

    async fn shutdown(&self) {
        match self {
            StorageBackend::Sqlite(s) => s.shutdown().await,
            StorageBackend::Redis(s) => s.shutdown().await,
            StorageBackend::Mysql(s) => s.shutdown().await,
        }
    }

    async fn get(&self, table: &str, key: &str) -> Result<Option<Vec<u8>>> {
        match self {
            StorageBackend::Sqlite(s) => s.get(table, key).await,
            StorageBackend::Redis(s) => s.get(table, key).await,
            StorageBackend::Mysql(s) => s.get(table, key).await,
        }
    }

    async fn set(&self, table: &str, key: &str, value: &[u8]) -> Result<()> {
        match self {
            StorageBackend::Sqlite(s) => s.set(table, key, value).await,
            StorageBackend::Redis(s) => s.set(table, key, value).await,
            StorageBackend::Mysql(s) => s.set(table, key, value).await,
        }
    }

    async fn delete(&self, table: &str, key: &str) -> Result<()> {
        match self {
            StorageBackend::Sqlite(s) => s.delete(table, key).await,
            StorageBackend::Redis(s) => s.delete(table, key).await,
            StorageBackend::Mysql(s) => s.delete(table, key).await,
        }
    }

    async fn exists(&self, table: &str, key: &str) -> Result<bool> {
        match self {
            StorageBackend::Sqlite(s) => s.exists(table, key).await,
            StorageBackend::Redis(s) => s.exists(table, key).await,
            StorageBackend::Mysql(s) => s.exists(table, key).await,
        }
    }

    async fn keys(&self, table: &str, pattern: &str) -> Result<Vec<String>> {
        match self {
            StorageBackend::Sqlite(s) => s.keys(table, pattern).await,
            StorageBackend::Redis(s) => s.keys(table, pattern).await,
            StorageBackend::Mysql(s) => s.keys(table, pattern).await,
        }
    }

    async fn query(
        &self,
        sql: &str,
        params: &[&dyn sqlx::Encode<'_, sqlx::Any>],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        match self {
            StorageBackend::Sqlite(s) => s.query(sql, params).await,
            StorageBackend::Redis(s) => s.query(sql, params).await,
            StorageBackend::Mysql(s) => s.query(sql, params).await,
        }
    }

    async fn execute(&self, sql: &str, params: &[&dyn sqlx::Encode<'_, sqlx::Any>]) -> Result<u64> {
        match self {
            StorageBackend::Sqlite(s) => s.execute(sql, params).await,
            StorageBackend::Redis(s) => s.execute(sql, params).await,
            StorageBackend::Mysql(s) => s.execute(sql, params).await,
        }
    }
}

pub struct StorageManager {
    backend: StorageBackend,
}

impl StorageManager {
    pub fn new(config: &crate::config::Config) -> Self {
        let backend = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                StorageBackend::new(&config.storage)
                    .await
                    .expect("Failed to create storage backend")
            })
        });
        Self { backend }
    }

    pub async fn init(&self) -> Result<()> {
        self.backend.init().await
    }

    pub async fn shutdown(&self) {
        self.backend.shutdown().await
    }

    pub fn backend(&self) -> &StorageBackend {
        &self.backend
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedData<T> {
    pub data: T,
    pub version: u32,
    pub timestamp: i64,
}

impl<T> SerializedData<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(data: T) -> Self {
        Self {
            data,
            version: 1,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
