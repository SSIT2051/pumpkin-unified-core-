use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warp {
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub creator: Uuid,
    pub created_at: i64,
}

pub struct WarpsModule {
    storage: Arc<StorageBackend>,
    cache: Arc<DashMap<String, Warp>>,
    config: Arc<crate::config::WarpsConfig>,
}

impl WarpsModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().warps);
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::WarpsConfig>,
    ) -> Self {
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn get_warp(&self, name: &str) -> Result<Option<Warp>> {
        if let Some(cached) = self.cache.get(name) {
            return Ok(Some(cached.clone()));
        }

        let key = name.to_lowercase();
        if let Some(data) = self.storage.get("warps", &key).await? {
            let warp_data: crate::storage::SerializedData<Warp> =
                crate::storage::SerializedData::deserialize(&data)?;
            self.cache.insert(key.clone(), warp_data.data.clone());
            return Ok(Some(warp_data.data));
        }
        Ok(None)
    }

    pub async fn set_warp(&self, warp: Warp) -> Result<()> {
        let key = warp.name.to_lowercase();
        let data = crate::storage::SerializedData::new(warp.clone()).serialize()?;
        self.storage.set("warps", &key, &data).await?;
        self.cache.insert(key, warp);
        Ok(())
    }

    pub async fn delete_warp(&self, name: &str) -> Result<()> {
        let key = name.to_lowercase();
        self.storage.delete("warps", &key).await?;
        self.cache.remove(&key);
        Ok(())
    }

    pub async fn list_warps(&self) -> Result<Vec<Warp>> {
        let keys = self.storage.keys("warps", "%").await?;
        let mut warps = Vec::new();
        for key in keys {
            if let Some(data) = self.storage.get("warps", &key).await? {
                let warp_data: crate::storage::SerializedData<Warp> =
                    crate::storage::SerializedData::deserialize(&data)?;
                warps.push(warp_data.data);
            }
        }
        Ok(warps)
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::warps::commands::register(self.clone(), context);
    }
}
