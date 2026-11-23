use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hologram {
    pub id: Uuid,
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub lines: Vec<String>,
    pub visibility_range: f64,
    pub page_lines: usize,
}

pub struct HologramsModule {
    storage: Arc<StorageBackend>,
    cache: Arc<DashMap<String, Hologram>>,
    config: Arc<crate::config::HologramsConfig>,
}

impl HologramsModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().holograms);
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(storage: Arc<StorageBackend>, config: Arc<crate::config::HologramsConfig>) -> Self {
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn get_hologram(&self, name: &str) -> Result<Option<Hologram>> {
        if let Some(cached) = self.cache.get(name) {
            return Ok(Some(cached.clone()));
        }

        let key = name.to_lowercase();
        if let Some(data) = self.storage.get("holograms", &key).await? {
            let holo_data: crate::storage::SerializedData<Hologram> = crate::storage::SerializedData::deserialize(&data)?;
            self.cache.insert(key.clone(), holo_data.data.clone());
            return Ok(Some(holo_data.data));
        }
        Ok(None)
    }

    pub async fn set_hologram(&self, hologram: Hologram) -> Result<()> {
        let key = hologram.name.to_lowercase();
        let data = crate::storage::SerializedData::new(hologram.clone()).serialize()?;
        self.storage.set("holograms", &key, &data).await?;
        self.cache.insert(key, hologram);
        Ok(())
    }

    pub async fn delete_hologram(&self, name: &str) -> Result<()> {
        let key = name.to_lowercase();
        self.storage.delete("holograms", &key).await?;
        self.cache.remove(&key);
        Ok(())
    }

    pub async fn list_holograms(&self) -> Result<Vec<Hologram>> {
        let keys = self.storage.keys("holograms", "%").await?;
        let mut holograms = Vec::new();
        for key in keys {
            if let Some(data) = self.storage.get("holograms", &key).await? {
                let holo_data: crate::storage::SerializedData<Hologram> = crate::storage::SerializedData::deserialize(&data)?;
                holograms.push(holo_data.data);
            }
        }
        Ok(holograms)
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::holograms::commands::register(self.clone(), context);
    }
}