use crate::storage::Storage;
use crate::utils::macros::*;
use anyhow::Result;
use sqlx::{Any, AnyPool, Pool, Row};
use std::collections::HashMap;
use std::sync::Arc;

pub struct MysqlStorage {
    pool: Arc<Pool<Any>>,
}

impl MysqlStorage {
    pub async fn new(config: &crate::config::MysqlConfig) -> Result<Self> {
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    async fn ensure_table(&self, table: &str) -> Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (key VARCHAR(255) PRIMARY KEY, value LONGBLOB NOT NULL, updated_at BIGINT NOT NULL)",
            table
        );
        sqlx::query(&sql).execute(self.pool.as_ref()).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Storage for MysqlStorage {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&self) {
        self.pool.close().await;
    }

    async fn get(&self, table: &str, key: &str) -> Result<Option<Vec<u8>>> {
        self.ensure_table(table).await?;
        let sql = format!("SELECT value FROM {} WHERE key = ?", table);
        let row = sqlx::query(&sql)
            .bind(key)
            .fetch_optional(self.pool.as_ref())
            .await?;

        Ok(row.map(|r| r.get::<Vec<u8>, _>("value")))
    }

    async fn set(&self, table: &str, key: &str, value: &[u8]) -> Result<()> {
        self.ensure_table(table).await?;
        let timestamp = chrono::Utc::now().timestamp();
        let sql = format!(
            "INSERT INTO {} (key, value, updated_at) VALUES (?, ?, ?)
             ON DUPLICATE KEY UPDATE value = VALUES(value), updated_at = VALUES(updated_at)",
            table
        );
        sqlx::query(&sql)
            .bind(key)
            .bind(value)
            .bind(timestamp)
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    async fn delete(&self, table: &str, key: &str) -> Result<()> {
        self.ensure_table(table).await?;
        let sql = format!("DELETE FROM {} WHERE key = ?", table);
        sqlx::query(&sql)
            .bind(key)
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    async fn exists(&self, table: &str, key: &str) -> Result<bool> {
        self.ensure_table(table).await?;
        let sql = format!("SELECT 1 FROM {} WHERE key = ? LIMIT 1", table);
        let row = sqlx::query(&sql)
            .bind(key)
            .fetch_optional(self.pool.as_ref())
            .await?;
        Ok(row.is_some())
    }

    async fn keys(&self, table: &str, pattern: &str) -> Result<Vec<String>> {
        self.ensure_table(table).await?;
        let sql = if pattern.contains('%') || pattern.contains('_') {
            format!("SELECT key FROM {} WHERE key LIKE ?", table)
        } else {
            format!("SELECT key FROM {} WHERE key = ?", table)
        };
        let rows = sqlx::query(&sql)
            .bind(pattern)
            .fetch_all(self.pool.as_ref())
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("key"))
            .collect())
    }

    async fn query(
        &self,
        sql: &str,
        params: &[&dyn sqlx::Encode<'_, sqlx::Any>],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let mut query = sqlx::query(sql);
        for param in params {
            query = query.bind(param);
        }
        let rows = query.fetch_all(self.pool.as_ref()).await?;

        let mut results = Vec::new();
        for row in rows {
            let mut map = HashMap::new();
            for column in row.columns() {
                let name = column.name().to_string();
                let value: serde_json::Value = row.try_get(name.clone())?;
                map.insert(name, value);
            }
            results.push(map);
        }
        Ok(results)
    }

    async fn execute(&self, sql: &str, params: &[&dyn sqlx::Encode<'_, sqlx::Any>]) -> Result<u64> {
        let mut query = sqlx::query(sql);
        for param in params {
            query = query.bind(param);
        }
        let result = query.execute(self.pool.as_ref()).await?;
        Ok(result.rows_affected())
    }
}
