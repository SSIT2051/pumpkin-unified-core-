use crate::storage::{SerializedData, StorageBackend};
use crate::utils::macros::*;
use crate::utils::text::format_text;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

const PLAYER_TRACKER_TABLE: &str = "playertracker";
const IP_HISTORY_TABLE: &str = "playertracker_ips";
const SESSION_TABLE: &str = "playertracker_sessions";
const AUDIT_LOG_TABLE: &str = "playertracker_audit";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTrackerData {
    pub uuid: Uuid,
    pub name: String,
    pub first_join: i64,
    pub last_seen: i64,
    pub total_playtime: i64,
    pub session_start: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpHistoryEntry {
    pub id: Uuid,
    pub uuid: Uuid,
    pub ip: String,
    pub first_seen: i64,
    pub last_seen: i64,
    pub join_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    pub id: Uuid,
    pub uuid: Uuid,
    pub login_time: i64,
    pub logout_time: Option<i64>,
    pub duration: Option<i64>,
    pub ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub uuid: Uuid,
    pub timestamp: i64,
    pub action: String,
    pub details: String,
    pub world: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct PlayerTrackerModule {
    storage: Arc<StorageBackend>,
    cache: Arc<DashMap<Uuid, PlayerTrackerData>>,
    ip_cache: Arc<DashMap<Uuid, Vec<IpHistoryEntry>>>,
    session_cache: Arc<DashMap<Uuid, Vec<SessionEntry>>>,
    audit_cache: Arc<DashMap<Uuid, Vec<AuditLogEntry>>>,
}

impl PlayerTrackerModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            ip_cache: Arc::new(DashMap::new()),
            session_cache: Arc::new(DashMap::new()),
            audit_cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn track_join(&self, uuid: Uuid, name: &str, ip: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let mut data = self.get_data(uuid).await?.unwrap_or_else(|| PlayerTrackerData {
            uuid,
            name: name.to_string(),
            first_join: now,
            last_seen: now,
            total_playtime: 0,
            session_start: None,
        });
        if data.first_join == 0 {
            data.first_join = now;
        }
        data.name = name.to_string();
        data.last_seen = now;
        data.session_start = Some(now);
        self.save_data(uuid, &data).await?;

        // Track IP
        self.track_ip(uuid, ip).await?;

        // Create session
        let session = SessionEntry {
            id: Uuid::new_v4(),
            uuid,
            login_time: now,
            logout_time: None,
            duration: None,
            ip: ip.to_string(),
        };
        self.save_session(uuid, &session).await?;

        Ok(())
    }

    pub async fn track_quit(&self, uuid: Uuid) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let mut data = match self.get_data(uuid).await? {
            Some(d) => d,
            None => return Ok(()),
        };
        if let Some(start) = data.session_start {
            data.total_playtime += now - start;
        }
        data.session_start = None;
        data.last_seen = now;
        self.save_data(uuid, &data).await?;

        // Update session
        self.end_session(uuid, now).await?;

        Ok(())
    }

    async fn track_ip(&self, uuid: Uuid, ip: &str) -> Result<()> {
        let key = format!("{}:{}", uuid, ip);
        let mut ips = self.ip_cache.get(&uuid).map(|v| v.clone()).unwrap_or_default();
        
        if let Some(entry) = ips.iter_mut().find(|e| e.ip == ip) {
            entry.last_seen = chrono::Utc::now().timestamp();
            entry.join_count += 1;
        } else {
            ips.push(IpHistoryEntry {
                id: Uuid::new_v4(),
                uuid,
                ip: ip.to_string(),
                first_seen: chrono::Utc::now().timestamp(),
                last_seen: chrono::Utc::now().timestamp(),
                join_count: 1,
            });
        }
        
        self.ip_cache.insert(uuid, ips.clone());
        // Save to storage
        let data = SerializedData::new(ips.clone()).serialize()?;
        self.storage.set(IP_HISTORY_TABLE, &uuid.to_string(), &data).await?;
        
        Ok(())
    }

    async fn save_session(&self, uuid: Uuid, session: &SessionEntry) -> Result<()> {
        let mut sessions = self.session_cache.get(&uuid).map(|v| v.clone()).unwrap_or_default();
        sessions.push(session.clone());
        self.session_cache.insert(uuid, sessions.clone());
        
        let data = SerializedData::new(sessions.clone()).serialize()?;
        self.storage.set(SESSION_TABLE, &uuid.to_string(), &data).await?;
        
        Ok(())
    }

    async fn end_session(&self, uuid: Uuid, logout_time: i64) -> Result<()> {
        let mut sessions = match self.session_cache.get_mut(&uuid) {
            Some(s) => s,
            None => return Ok(()),
        };
        
        if let Some(session) = sessions.iter_mut().find(|s| s.logout_time.is_none()) {
            session.logout_time = Some(logout_time);
            session.duration = Some(logout_time - session.login_time);
        }
        
        let data = SerializedData::new(sessions.clone()).serialize()?;
        self.storage.set(SESSION_TABLE, &uuid.to_string(), &data).await?;
        
        Ok(())
    }

    pub async fn log_audit(&self, uuid: Uuid, action: &str, details: &str, world: &str, x: i32, y: i32, z: i32) -> Result<()> {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            uuid,
            timestamp: chrono::Utc::now().timestamp(),
            action: action.to_string(),
            details: details.to_string(),
            world: world.to_string(),
            x,
            y,
            z,
        };
        
        let mut audits = self.audit_cache.get(&uuid).map(|v| v.clone()).unwrap_or_default();
        audits.push(entry.clone());
        
        // Keep only max entries
        if audits.len() > 1000 {
            audits.drain(0..audits.len() - 1000);
        }
        
        self.audit_cache.insert(uuid, audits.clone());
        
        let data = SerializedData::new(audits.clone()).serialize()?;
        self.storage.set(AUDIT_LOG_TABLE, &uuid.to_string(), &data).await?;
        
        Ok(())
    }

    pub async fn get_data(&self, uuid: Uuid) -> Result<Option<PlayerTrackerData>> {
        if let Some(cached) = self.cache.get(&uuid) {
            return Ok(Some(cached.clone()));
        }

        let key = uuid.to_string();
        if let Some(data) = self.storage.get(PLAYER_TRACKER_TABLE, &key).await? {
            let tracker_data: SerializedData<PlayerTrackerData> =
                SerializedData::deserialize(&data)?;
            self.cache.insert(uuid, tracker_data.data.clone());
            return Ok(Some(tracker_data.data));
        }
        Ok(None)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<PlayerTrackerData>> {
        for entry in self.cache.iter() {
            if entry.value().name.eq_ignore_ascii_case(name) {
                return Ok(Some(entry.value().clone()));
            }
        }

        let keys = self.storage.keys(PLAYER_TRACKER_TABLE, "%").await?;
        for key in keys {
            if let Ok(uuid) = Uuid::parse_str(&key) {
                if let Some(data) = self.get_data(uuid).await? {
                    if data.name.eq_ignore_ascii_case(name) {
                        return Ok(Some(data));
                    }
                }
            }
        }
        Ok(None)
    }

    pub async fn get_playtime(&self, uuid: Uuid) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        match self.get_data(uuid).await? {
            Some(d) => {
                let mut playtime = d.total_playtime;
                if let Some(start) = d.session_start {
                    playtime += now - start;
                }
                Ok(playtime)
            }
            None => Ok(0),
        }
    }

    pub async fn get_ip_history(&self, uuid: Uuid) -> Result<Vec<IpHistoryEntry>> {
        if let Some(cached) = self.ip_cache.get(&uuid) {
            return Ok(cached.clone());
        }

        let key = uuid.to_string();
        if let Some(data) = self.storage.get(IP_HISTORY_TABLE, &key).await? {
            let ip_data: SerializedData<Vec<IpHistoryEntry>> = SerializedData::deserialize(&data)?;
            self.ip_cache.insert(uuid, ip_data.data.clone());
            return Ok(ip_data.data);
        }
        Ok(Vec::new())
    }

    pub async fn get_sessions(&self, uuid: Uuid) -> Result<Vec<SessionEntry>> {
        if let Some(cached) = self.session_cache.get(&uuid) {
            return Ok(cached.clone());
        }

        let key = uuid.to_string();
        if let Some(data) = self.storage.get(SESSION_TABLE, &key).await? {
            let session_data: SerializedData<Vec<SessionEntry>> = SerializedData::deserialize(&data)?;
            self.session_cache.insert(uuid, session_data.data.clone());
            return Ok(session_data.data);
        }
        Ok(Vec::new())
    }

    pub async fn get_audit_log(&self, uuid: Uuid, action_filter: Option<&str>) -> Result<Vec<AuditLogEntry>> {
        if let Some(cached) = self.audit_cache.get(&uuid) {
            let mut audits = cached.clone();
            if let Some(filter) = action_filter {
                audits.retain(|a| a.action == filter);
            }
            return Ok(audits);
        }

        let key = uuid.to_string();
        if let Some(data) = self.storage.get(AUDIT_LOG_TABLE, &key).await? {
            let audit_data: SerializedData<Vec<AuditLogEntry>> = SerializedData::deserialize(&data)?;
            let mut audits = audit_data.data;
            if let Some(filter) = action_filter {
                audits.retain(|a| a.action == filter);
            }
            self.audit_cache.insert(uuid, audits.clone());
            return Ok(audits);
        }
        Ok(Vec::new())
    }

    async fn save_data(&self, uuid: Uuid, data: &PlayerTrackerData) -> Result<()> {
        let key = uuid.to_string();
        let serialized = SerializedData::new(data.clone()).serialize()?;
        self.storage
            .set(PLAYER_TRACKER_TABLE, &key, &serialized)
            .await?;
        self.cache.insert(uuid, data.clone());
        Ok(())
    }

    pub fn format_duration(seconds: i64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    pub fn format_ago(timestamp: i64) -> String {
        let now = chrono::Utc::now().timestamp();
        let diff = now.saturating_sub(timestamp);
        if diff < 60 {
            "just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::playertracker::commands::register(self.clone(), context);
    }
}

impl Clone for PlayerTrackerModule {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            cache: self.cache.clone(),
            ip_cache: self.ip_cache.clone(),
            session_cache: self.session_cache.clone(),
            audit_cache: self.audit_cache.clone(),
        }
    }
}