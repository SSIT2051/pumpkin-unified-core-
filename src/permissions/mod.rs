use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionNode {
    pub node: String,
    pub value: bool,
    pub server: Option<String>,
    pub world: Option<String>,
    pub expiry: Option<i64>,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub uuid: Uuid,
    pub name: String,
    pub permissions: Vec<PermissionNode>,
    pub groups: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupData {
    pub name: String,
    pub permissions: Vec<PermissionNode>,
    pub parents: Vec<String>,
    pub weight: i32,
    pub display_name: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackData {
    pub name: String,
    pub groups: Vec<String>,
}

pub struct PermissionsModule {
    storage: Arc<StorageBackend>,
    users: Arc<DashMap<Uuid, UserData>>,
    groups: Arc<DashMap<String, GroupData>>,
    tracks: Arc<DashMap<String, TrackData>>,
    config: Arc<crate::config::PermissionsConfig>,
}

impl PermissionsModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().permissions);
        Self {
            storage,
            users: Arc::new(DashMap::new()),
            groups: Arc::new(DashMap::new()),
            tracks: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::PermissionsConfig>,
    ) -> Self {
        Self {
            storage,
            users: Arc::new(DashMap::new()),
            groups: Arc::new(DashMap::new()),
            tracks: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn load_data(&self) -> Result<()> {
        for key in self.storage.keys("users", "%").await? {
            if let Some(data) = self.storage.get("users", &key).await? {
                let entry: crate::storage::SerializedData<UserData> =
                    crate::storage::SerializedData::deserialize(&data)?;
                self.users.insert(entry.data.uuid, entry.data);
            }
        }
        for key in self.storage.keys("groups", "%").await? {
            if let Some(data) = self.storage.get("groups", &key).await? {
                let entry: crate::storage::SerializedData<GroupData> =
                    crate::storage::SerializedData::deserialize(&data)?;
                self.groups.insert(entry.data.name.clone(), entry.data);
            }
        }
        for key in self.storage.keys("tracks", "%").await? {
            if let Some(data) = self.storage.get("tracks", &key).await? {
                let entry: crate::storage::SerializedData<TrackData> =
                    crate::storage::SerializedData::deserialize(&data)?;
                self.tracks.insert(entry.data.name.clone(), entry.data);
            }
        }

        if !self.groups.contains_key(&self.config.default_group) {
            self.create_group(self.config.default_group.clone()).await?;
        }

        Ok(())
    }

    pub async fn create_group(&self, name: String) -> Result<()> {
        let group = GroupData {
            name: name.clone(),
            permissions: vec![],
            parents: vec![],
            weight: 0,
            display_name: None,
            metadata: HashMap::new(),
        };
        let data = crate::storage::SerializedData::new(group.clone()).serialize()?;
        self.storage.set("groups", &name, &data).await?;
        self.groups.insert(name, group);
        Ok(())
    }

    pub fn get_user(&self, uuid: Uuid) -> Option<UserData> {
        self.users.get(&uuid).map(|u| u.clone())
    }

    pub fn get_group(&self, name: &str) -> Option<GroupData> {
        self.groups.get(name).map(|g| g.clone())
    }

    pub fn has_permission(
        &self,
        uuid: Uuid,
        permission: &str,
        contexts: &HashMap<String, String>,
    ) -> bool {
        let user = match self.get_user(uuid) {
            Some(u) => u,
            None => return false,
        };

        let mut all_perms = user.permissions.clone();

        for group_name in &user.groups {
            if let Some(group) = self.get_group(group_name) {
                all_perms.extend(group.permissions);
                for parent in &group.parents {
                    if let Some(parent_group) = self.get_group(parent) {
                        all_perms.extend(parent_group.permissions);
                    }
                }
            }
        }

        for perm in all_perms {
            if perm.node == permission || self.wildcard_match(&perm.node, permission) {
                if !perm.value {
                    return false;
                }
                if let Some(expiry) = perm.expiry {
                    if chrono::Utc::now().timestamp() > expiry {
                        continue;
                    }
                }
                if !self.contexts_match(&perm.context, contexts) {
                    return false;
                }
                return true;
            }
        }
        false
    }

    fn wildcard_match(&self, pattern: &str, permission: &str) -> bool {
        if pattern.ends_with(".*") {
            let prefix = &pattern[..pattern.len() - 1];
            permission.starts_with(prefix)
        } else {
            false
        }
    }

    fn contexts_match(
        &self,
        perm_contexts: &HashMap<String, String>,
        check_contexts: &HashMap<String, String>,
    ) -> bool {
        for (key, value) in perm_contexts {
            if let Some(check_value) = check_contexts.get(key) {
                if check_value != value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub async fn set_user_permission(
        &self,
        uuid: Uuid,
        node: String,
        value: bool,
        contexts: HashMap<String, String>,
    ) -> Result<()> {
        let mut user = self.get_user(uuid).unwrap_or_else(|| UserData {
            uuid,
            name: String::new(),
            permissions: vec![],
            groups: vec![],
            metadata: HashMap::new(),
        });

        user.permissions
            .retain(|p| p.node != node || p.context != contexts);
        user.permissions.push(PermissionNode {
            node,
            value,
            server: None,
            world: None,
            expiry: None,
            context: contexts,
        });

        let data = crate::storage::SerializedData::new(user.clone()).serialize()?;
        self.storage.set("users", &uuid.to_string(), &data).await?;
        self.users.insert(uuid, user);
        Ok(())
    }

    pub async fn add_user_group(&self, uuid: Uuid, group: String) -> Result<()> {
        let mut user = self.get_user(uuid).unwrap_or_else(|| UserData {
            uuid,
            name: String::new(),
            permissions: vec![],
            groups: vec![],
            metadata: HashMap::new(),
        });

        if !user.groups.contains(&group) {
            user.groups.push(group);
            let data = crate::storage::SerializedData::new(user.clone()).serialize()?;
            self.storage.set("users", &uuid.to_string(), &data).await?;
            self.users.insert(uuid, user);
        }
        Ok(())
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::permissions::commands::register(self.clone(), context);
    }
}
