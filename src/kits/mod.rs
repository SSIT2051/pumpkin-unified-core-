use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitItem {
    pub item_type: String,
    pub amount: u32,
    pub data: Option<u16>,
    pub enchantments: Vec<Enchantment>,
    pub name: Option<String>,
    pub lore: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enchantment {
    pub enchantment: String,
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kit {
    pub name: String,
    pub display_name: String,
    pub items: Vec<KitItem>,
    pub cooldown: u64,
    pub permission: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitCooldown {
    pub uuid: Uuid,
    pub kit_name: String,
    pub last_used: i64,
}

pub struct KitsModule {
    storage: Arc<StorageBackend>,
    kits: Arc<DashMap<String, Kit>>,
    cooldowns: Arc<DashMap<String, KitCooldown>>,
    config: Arc<crate::config::KitsConfig>,
}

impl KitsModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().kits);
        Self {
            storage,
            kits: Arc::new(DashMap::new()),
            cooldowns: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::KitsConfig>,
    ) -> Self {
        Self {
            storage,
            kits: Arc::new(DashMap::new()),
            cooldowns: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn load_kits(&self) -> Result<()> {
        let keys = self.storage.keys("kits", "%").await?;
        for key in keys {
            if let Some(data) = self.storage.get("kits", &key).await? {
                let kit_data: crate::storage::SerializedData<Kit> =
                    crate::storage::SerializedData::deserialize(&data)?;
                self.kits.insert(key, kit_data.data);
            }
        }
        Ok(())
    }

    pub async fn get_kit(&self, name: &str) -> Option<Kit> {
        self.kits.get(name).map(|k| k.clone())
    }

    pub async fn set_kit(&self, kit: Kit) -> Result<()> {
        let key = kit.name.to_lowercase();
        let data = crate::storage::SerializedData::new(kit.clone()).serialize()?;
        self.storage.set("kits", &key, &data).await?;
        self.kits.insert(key, kit);
        Ok(())
    }

    pub async fn delete_kit(&self, name: &str) -> Result<()> {
        let key = name.to_lowercase();
        self.storage.delete("kits", &key).await?;
        self.kits.remove(&key);
        Ok(())
    }

    pub async fn list_kits(&self) -> Vec<Kit> {
        self.kits.iter().map(|k| k.clone()).collect()
    }

    pub fn check_cooldown(&self, uuid: Uuid, kit_name: &str) -> Option<u64> {
        let key = format!("{}:{}", uuid, kit_name.to_lowercase());
        if let Some(cooldown) = self.cooldowns.get(&key) {
            let now = chrono::Utc::now().timestamp();
            let elapsed = now - cooldown.last_used;
            let kit = self.kits.get(kit_name)?;
            if elapsed < kit.cooldown as i64 {
                return Some((kit.cooldown as i64 - elapsed) as u64);
            }
        }
        None
    }

    pub fn set_cooldown(&self, uuid: Uuid, kit_name: &str) {
        let key = format!("{}:{}", uuid, kit_name.to_lowercase());
        self.cooldowns.insert(
            key,
            KitCooldown {
                uuid,
                kit_name: kit_name.to_string(),
                last_used: chrono::Utc::now().timestamp(),
            },
        );
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::kits::commands::register(self.clone(), context);
    }
}
