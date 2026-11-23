use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanEntry {
    pub uuid: Uuid,
    pub name: String,
    pub reason: String,
    pub banned_by: Uuid,
    pub banned_at: i64,
    pub expires_at: Option<i64>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuteEntry {
    pub uuid: Uuid,
    pub name: String,
    pub reason: String,
    pub muted_by: Uuid,
    pub muted_at: i64,
    pub expires_at: Option<i64>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JailEntry {
    pub uuid: Uuid,
    pub name: String,
    pub reason: String,
    pub jailed_by: Uuid,
    pub jailed_at: i64,
    pub expires_at: Option<i64>,
    pub active: bool,
}

pub struct ModerationModule {
    storage: Arc<StorageBackend>,
    bans: Arc<DashMap<Uuid, BanEntry>>,
    mutes: Arc<DashMap<Uuid, MuteEntry>>,
    jails: Arc<DashMap<Uuid, JailEntry>>,
    config: Arc<crate::config::ModerationConfig>,
}

impl ModerationModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().moderation);
        Self {
            storage,
            bans: Arc::new(DashMap::new()),
            mutes: Arc::new(DashMap::new()),
            jails: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::ModerationConfig>,
    ) -> Self {
        Self {
            storage,
            bans: Arc::new(DashMap::new()),
            mutes: Arc::new(DashMap::new()),
            jails: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn load_data(&self) -> Result<()> {
        for key in self.storage.keys("bans", "%").await? {
            if let Some(data) = self.storage.get("bans", &key).await? {
                let entry: crate::storage::SerializedData<BanEntry> =
                    crate::storage::SerializedData::deserialize(&data)?;
                if entry.data.active {
                    self.bans.insert(entry.data.uuid, entry.data);
                }
            }
        }
        for key in self.storage.keys("mutes", "%").await? {
            if let Some(data) = self.storage.get("mutes", &key).await? {
                let entry: crate::storage::SerializedData<MuteEntry> =
                    crate::storage::SerializedData::deserialize(&data)?;
                if entry.data.active {
                    self.mutes.insert(entry.data.uuid, entry.data);
                }
            }
        }
        for key in self.storage.keys("jails", "%").await? {
            if let Some(data) = self.storage.get("jails", &key).await? {
                let entry: crate::storage::SerializedData<JailEntry> =
                    crate::storage::SerializedData::deserialize(&data)?;
                if entry.data.active {
                    self.jails.insert(entry.data.uuid, entry.data);
                }
            }
        }
        Ok(())
    }

    pub fn is_banned(&self, uuid: Uuid) -> Option<BanEntry> {
        self.bans
            .get(&uuid)
            .filter(|b| {
                b.active
                    && b.expires_at
                        .map_or(true, |e| e > chrono::Utc::now().timestamp())
            })
            .map(|b| b.clone())
    }

    pub fn is_muted(&self, uuid: Uuid) -> Option<MuteEntry> {
        self.mutes
            .get(&uuid)
            .filter(|m| {
                m.active
                    && m.expires_at
                        .map_or(true, |e| e > chrono::Utc::now().timestamp())
            })
            .map(|m| m.clone())
    }

    pub fn is_jailed(&self, uuid: Uuid) -> Option<JailEntry> {
        self.jails
            .get(&uuid)
            .filter(|j| {
                j.active
                    && j.expires_at
                        .map_or(true, |e| e > chrono::Utc::now().timestamp())
            })
            .map(|j| j.clone())
    }

    pub async fn ban(
        &self,
        target: Uuid,
        name: String,
        reason: String,
        by: Uuid,
        duration: Option<u64>,
    ) -> Result<()> {
        let expires_at = duration.map(|d| chrono::Utc::now().timestamp() + d as i64);
        let entry = BanEntry {
            uuid: target,
            name,
            reason,
            banned_by: by,
            banned_at: chrono::Utc::now().timestamp(),
            expires_at,
            active: true,
        };
        let data = crate::storage::SerializedData::new(entry.clone()).serialize()?;
        self.storage.set("bans", &target.to_string(), &data).await?;
        self.bans.insert(target, entry);
        Ok(())
    }

    pub async fn unban(&self, target: Uuid) -> Result<()> {
        if let Some(mut entry) = self.bans.get_mut(&target) {
            entry.active = false;
            let data = crate::storage::SerializedData::new(entry.clone()).serialize()?;
            self.storage.set("bans", &target.to_string(), &data).await?;
        }
        Ok(())
    }

    pub async fn mute(
        &self,
        target: Uuid,
        name: String,
        reason: String,
        by: Uuid,
        duration: Option<u64>,
    ) -> Result<()> {
        let expires_at = duration.map(|d| chrono::Utc::now().timestamp() + d as i64);
        let entry = MuteEntry {
            uuid: target,
            name,
            reason,
            muted_by: by,
            muted_at: chrono::Utc::now().timestamp(),
            expires_at,
            active: true,
        };
        let data = crate::storage::SerializedData::new(entry.clone()).serialize()?;
        self.storage
            .set("mutes", &target.to_string(), &data)
            .await?;
        self.mutes.insert(target, entry);
        Ok(())
    }

    pub async fn unmute(&self, target: Uuid) -> Result<()> {
        if let Some(mut entry) = self.mutes.get_mut(&target) {
            entry.active = false;
            let data = crate::storage::SerializedData::new(entry.clone()).serialize()?;
            self.storage
                .set("mutes", &target.to_string(), &data)
                .await?;
        }
        Ok(())
    }

    pub async fn jail(
        &self,
        target: Uuid,
        name: String,
        reason: String,
        by: Uuid,
        duration: Option<u64>,
    ) -> Result<()> {
        let expires_at = duration.map(|d| chrono::Utc::now().timestamp() + d as i64);
        let entry = JailEntry {
            uuid: target,
            name,
            reason,
            jailed_by: by,
            jailed_at: chrono::Utc::now().timestamp(),
            expires_at,
            active: true,
        };
        let data = crate::storage::SerializedData::new(entry.clone()).serialize()?;
        self.storage
            .set("jails", &target.to_string(), &data)
            .await?;
        self.jails.insert(target, entry);
        Ok(())
    }

    pub async fn unjail(&self, target: Uuid) -> Result<()> {
        if let Some(mut entry) = self.jails.get_mut(&target) {
            entry.active = false;
            let data = crate::storage::SerializedData::new(entry.clone()).serialize()?;
            self.storage
                .set("jails", &target.to_string(), &data)
                .await?;
        }
        Ok(())
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::moderation::commands::register(self.clone(), context);
    }
}
