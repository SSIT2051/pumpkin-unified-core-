use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portal {
    pub id: Uuid,
    pub name: String,
    pub world: String,
    pub x1: f64,
    pub y1: f64,
    pub z1: f64,
    pub x2: f64,
    pub y2: f64,
    pub z2: f64,
    pub destination: Option<Destination>,
    pub command: Option<String>,
    pub particle: Option<String>,
    pub material: Option<String>,
    pub cooldown: i64,
    pub permission: Option<String>,
}

#[derive(Clone)]
pub struct PortalsModule {
    storage: Arc<StorageBackend>,
    cache: Arc<DashMap<String, Portal>>,
}

impl PortalsModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn get_portal(&self, name: &str) -> Result<Option<Portal>> {
        if let Some(cached) = self.cache.get(name) {
            return Ok(Some(cached.clone()));
        }
        let key = name.to_lowercase();
        if let Some(data) = self.storage.get("portals", &key).await? {
            let portal_data: crate::storage::SerializedData<Portal> =
                crate::storage::SerializedData::deserialize(&data)?;
            self.cache.insert(key.clone(), portal_data.data.clone());
            return Ok(Some(portal_data.data));
        }
        Ok(None)
    }

    pub async fn set_portal(&self, portal: Portal) -> Result<()> {
        let key = portal.name.to_lowercase();
        let data = crate::storage::SerializedData::new(portal.clone()).serialize()?;
        self.storage.set("portals", &key, &data).await?;
        self.cache.insert(key, portal);
        Ok(())
    }

    pub async fn delete_portal(&self, name: &str) -> Result<()> {
        let key = name.to_lowercase();
        self.storage.delete("portals", &key).await?;
        self.cache.remove(&key);
        Ok(())
    }

    pub async fn list_portals(&self) -> Result<Vec<Portal>> {
        let keys = self.storage.keys("portals", "%").await?;
        let mut portals = Vec::new();
        for key in keys {
            if let Some(data) = self.storage.get("portals", &key).await? {
                let portal_data: crate::storage::SerializedData<Portal> =
                    crate::storage::SerializedData::deserialize(&data)?;
                portals.push(portal_data.data);
            }
        }
        Ok(portals)
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::portals::commands::register(self.clone(), context);
    }
}
