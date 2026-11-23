use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpaRequest {
    pub from: Uuid,
    pub to: Uuid,
    pub request_type: TpaType,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TpaType {
    Tpa,
    Tpahere,
}

pub struct TpaModule {
    requests: Arc<DashMap<Uuid, TpaRequest>>,
    toggles: Arc<DashMap<Uuid, bool>>,
    config: Arc<crate::config::TpaConfig>,
}

impl TpaModule {
    pub fn new() -> Self {
        let config = Arc::new(crate::config::Config::default().tpa);
        Self {
            requests: Arc::new(DashMap::new()),
            toggles: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(config: Arc<crate::config::TpaConfig>) -> Self {
        Self {
            requests: Arc::new(DashMap::new()),
            toggles: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn create_request(&self, from: Uuid, to: Uuid, request_type: TpaType) -> Result<()> {
        if !self.toggles.get(&to).copied().unwrap_or(true) {
            return Err(anyhow::anyhow!("Player has TPA toggled off"));
        }

        let now = chrono::Utc::now().timestamp();
        let request = TpaRequest {
            from,
            to,
            request_type,
            created_at: now,
            expires_at: now + self.config.request_timeout as i64,
        };

        self.requests.insert(to, request);
        Ok(())
    }

    pub fn get_request(&self, to: Uuid) -> Option<TpaRequest> {
        self.requests.get(&to).map(|r| r.clone())
    }

    pub fn remove_request(&self, to: Uuid) {
        self.requests.remove(&to);
    }

    pub fn is_expired(&self, request: &TpaRequest) -> bool {
        chrono::Utc::now().timestamp() > request.expires_at
    }

    pub fn set_toggle(&self, uuid: Uuid, enabled: bool) {
        self.toggles.insert(uuid, enabled);
    }

    pub fn get_toggle(&self, uuid: Uuid) -> bool {
        self.toggles.get(&uuid).copied().unwrap_or(false)
    }

    pub fn cleanup_expired(&self) {
        let now = chrono::Utc::now().timestamp();
        self.requests.retain(|_, v| v.expires_at > now);
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::tpa::commands::register(self.clone(), context);
    }
}
