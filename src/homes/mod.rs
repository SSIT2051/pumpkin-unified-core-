use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Home {
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub created_at: i64,
}

pub struct HomesModule {
    storage: Arc<StorageBackend>,
    cache: Arc<DashMap<String, Home>>,
    config: Arc<crate::config::HomesConfig>,
}

impl HomesModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().homes);
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::HomesConfig>,
    ) -> Self {
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    fn cache_key(&self, uuid: Uuid, name: &str) -> String {
        format!("{}:{}", uuid, name.to_lowercase())
    }

    pub async fn get_home(&self, uuid: Uuid, name: &str) -> Result<Option<Home>> {
        let key = self.cache_key(uuid, name);
        if let Some(cached) = self.cache.get(&key) {
            return Ok(Some(cached.clone()));
        }

        if let Some(data) = self.storage.get("homes", &key).await? {
            let home_data: crate::storage::SerializedData<Home> =
                crate::storage::SerializedData::deserialize(&data)?;
            self.cache.insert(key.clone(), home_data.data.clone());
            return Ok(Some(home_data.data));
        }
        Ok(None)
    }

    pub async fn set_home(&self, uuid: Uuid, home: Home) -> Result<()> {
        let key = self.cache_key(uuid, &home.name);
        let data = crate::storage::SerializedData::new(home.clone()).serialize()?;
        self.storage.set("homes", &key, &data).await?;
        self.cache.insert(key, home);
        Ok(())
    }

    pub async fn delete_home(&self, uuid: Uuid, name: &str) -> Result<()> {
        let key = self.cache_key(uuid, name);
        self.storage.delete("homes", &key).await?;
        self.cache.remove(&key);
        Ok(())
    }

    pub async fn list_homes(&self, uuid: Uuid) -> Result<Vec<Home>> {
        let pattern = format!("{}:*", uuid);
        let keys = self.storage.keys("homes", &pattern).await?;
        let mut homes = Vec::new();
        for key in keys {
            if let Some(data) = self.storage.get("homes", &key).await? {
                let home_data: crate::storage::SerializedData<Home> =
                    crate::storage::SerializedData::deserialize(&data)?;
                homes.push(home_data.data);
            }
        }
        Ok(homes)
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::homes::commands::register(self.clone(), context);
    }
}
