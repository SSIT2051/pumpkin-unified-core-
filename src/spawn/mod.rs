use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnData {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

pub struct SpawnModule {
    storage: Arc<StorageBackend>,
    spawn: Arc<RwLock<Option<SpawnData>>>,
    config: Arc<crate::config::SpawnConfig>,
}

impl SpawnModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().spawn);
        Self {
            storage,
            spawn: Arc::new(RwLock::new(None)),
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::SpawnConfig>,
    ) -> Self {
        Self {
            storage,
            spawn: Arc::new(RwLock::new(None)),
            config,
        }
    }

    pub async fn load_spawn(&self) -> Result<()> {
        if let Some(data) = self.storage.get("spawn", "main").await? {
            let spawn_data: crate::storage::SerializedData<SpawnData> =
                crate::storage::SerializedData::deserialize(&data)?;
            *self.spawn.write().unwrap() = Some(spawn_data.data);
        } else if let Some(loc) = &self.config.spawn_location {
            *self.spawn.write().unwrap() = Some(SpawnData {
                world: loc.world.clone(),
                x: loc.x,
                y: loc.y,
                z: loc.z,
                yaw: loc.yaw,
                pitch: loc.pitch,
            });
        }
        Ok(())
    }

    pub async fn set_spawn(&self, spawn: SpawnData) -> Result<()> {
        let data = crate::storage::SerializedData::new(spawn.clone()).serialize()?;
        self.storage.set("spawn", "main", &data).await?;
        *self.spawn.write().unwrap() = Some(spawn);
        Ok(())
    }

    pub fn get_spawn(&self) -> Option<SpawnData> {
        self.spawn.read().unwrap().clone()
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::spawn::commands::register(self.clone(), context);
    }
}

impl Clone for SpawnModule {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            spawn: self.spawn.clone(),
            config: self.config.clone(),
        }
    }
}
