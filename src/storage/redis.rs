use crate::storage::Storage;
use crate::utils::macros::*;
use anyhow::Result;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use std::collections::HashMap;
use std::sync::Arc;

pub struct RedisStorage {
    conn: Arc<ConnectionManager>,
}

impl RedisStorage {
    pub async fn new(config: &crate::config::RedisConfig) -> Result<Self> {
        let client = Client::open(config.url.as_str())?;
        let conn = ConnectionManager::new(client).await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    fn key(&self, table: &str, key: &str) -> String {
        format!("puc:{}:{}", table, key)
    }
}

#[async_trait::async_trait]
impl Storage for RedisStorage {
    async fn init(&self) -> Result<()> {
        let mut conn = (*self.conn).clone();
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await?;
        Ok(())
    }

    async fn shutdown(&self) {}

    async fn get(&self, table: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = (*self.conn).clone();
        let value: Option<Vec<u8>> = conn.get(self.key(table, key)).await?;
        Ok(value)
    }

    async fn set(&self, table: &str, key: &str, value: &[u8]) -> Result<()> {
        let mut conn = (*self.conn).clone();
        conn.set(self.key(table, key), value).await?;
        Ok(())
    }

    async fn delete(&self, table: &str, key: &str) -> Result<()> {
        let mut conn = (*self.conn).clone();
        conn.del(self.key(table, key)).await?;
        Ok(())
    }

    async fn exists(&self, table: &str, key: &str) -> Result<bool> {
        let mut conn = (*self.conn).clone();
        let exists: bool = conn.exists(self.key(table, key)).await?;
        Ok(exists)
    }

    async fn keys(&self, table: &str, pattern: &str) -> Result<Vec<String>> {
        let mut conn = (*self.conn).clone();
        let search_pattern = self.key(table, pattern);
        let keys: Vec<String> = conn.keys(search_pattern).await?;
        Ok(keys
            .into_iter()
            .map(|k| {
                k.strip_prefix(&format!("puc:{}:", table))
                    .unwrap_or(&k)
                    .to_string()
            })
            .collect())
    }

    async fn query(
        &self,
        _sql: &str,
        _params: &[&dyn sqlx::Encode<'_, sqlx::Any>],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        Err(anyhow::anyhow!(
            "Raw SQL queries not supported in Redis backend"
        ))
    }

    async fn execute(
        &self,
        _sql: &str,
        _params: &[&dyn sqlx::Encode<'_, sqlx::Any>],
    ) -> Result<u64> {
        Err(anyhow::anyhow!(
            "Raw SQL execute not supported in Redis backend"
        ))
    }
}
