use crate::storage::{StorageBackend, SerializedData};
use crate::utils::macros::*;
use crate::utils::text::{format_text, replace_placeholders};
use anyhow::Result;
use dashmap::DashMap;
use pumpkin_plugin_api::{
    context::Context,
    event::{AsyncPlayerChatEvent, EventHandler, EventPriority, PlayerJoinEvent},
    server::Server,
    text::TextComponent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChannel {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub color: String,
    pub format: String,
    pub permission: Option<String>,
    pub members: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerChatData {
    pub uuid: Uuid,
    pub channel: String,
    pub last_message: i64,
    pub message_count: u32,
}

pub struct ChatTabModule {
    storage: Arc<StorageBackend>,
    channels: Arc<DashMap<String, ChatChannel>>,
    player_data: Arc<DashMap<Uuid, PlayerChatData>>,
    config: Arc<crate::config::ChatTabConfig>,
}

impl ChatTabModule {
    pub fn new(config: Arc<crate::config::ChatTabConfig>) -> Self {
        Self {
            storage: Arc::new(StorageBackend::new()), // Will be replaced in with_config
            channels: Arc::new(DashMap::new()),
            player_data: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(storage: Arc<StorageBackend>, config: Arc<crate::config::ChatTabConfig>) -> Self {
        Self {
            storage,
            channels: Arc::new(DashMap::new()),
            player_data: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        self.load_channels().await?;
        
        // Create default channels
        self.create_default_channels().await?;
        
        Ok(())
    }

    async fn create_default_channels(&self) -> Result<()> {
        let defaults = vec![
            ("global", "Global", "&f", "{channel} {player}: {message}"),
            ("local", "Local", "&a", "[Local] {player}: {message}"),
            ("staff", "Staff", "&c", "[Staff] {player}: {message}"),
        ];

        for (name, display, color, format) in defaults {
            if !self.channels.contains_key(name) {
                let channel = ChatChannel {
                    id: Uuid::new_v4(),
                    name: name.to_string(),
                    display_name: display.to_string(),
                    color: color.to_string(),
                    format: format.to_string(),
                    permission: if name == "staff" { Some("pumpkin-unified-core.chattab.channel.staff".to_string()) } else { None },
                    members: Vec::new(),
                };
                self.channels.insert(name.to_string(), channel.clone());
                self.save_channel(&channel).await?;
            }
        }
        Ok(())
    }

    async fn load_channels(&self) -> Result<()> {
        let keys = self.storage.keys("chattab_channels", "%").await?;
        for key in keys {
            if let Some(data) = self.storage.get("chattab_channels", &key).await? {
                let channel_data: SerializedData<ChatChannel> = SerializedData::deserialize(&data)?;
                self.channels.insert(key, channel_data.data);
            }
        }
        Ok(())
    }

    async fn save_channel(&self, channel: &ChatChannel) -> Result<()> {
        let data = SerializedData::new(channel.clone()).serialize()?;
        self.storage.set("chattab_channels", &channel.name.to_lowercase(), &data).await?;
        Ok(())
    }

    pub fn get_channel(&self, name: &str) -> Option<ChatChannel> {
        self.channels.get(name).map(|c| c.clone())
    }

    pub fn list_channels(&self) -> Vec<ChatChannel> {
        self.channels.iter().map(|c| c.clone()).collect()
    }

    pub fn set_player_channel(&self, uuid: Uuid, channel: &str) {
        self.player_data.insert(uuid, PlayerChatData {
            uuid,
            channel: channel.to_string(),
            last_message: chrono::Utc::now().timestamp(),
            message_count: 0,
        });
    }

    pub fn get_player_channel(&self, uuid: Uuid) -> String {
        self.player_data.get(&uuid)
            .map(|d| d.channel.clone())
            .unwrap_or_else(|| "global".to_string())
    }

    pub fn format_chat_message(&self, channel_name: &str, player_name: &str, message: &str, group: &str, prefix: &str) -> String {
        let channel = self.get_channel(channel_name).unwrap_or_else(|| ChatChannel {
            id: Uuid::new_v4(),
            name: channel_name.to_string(),
            display_name: channel_name.to_string(),
            color: "&f".to_string(),
            format: "{channel} {player}: {message}".to_string(),
            permission: None,
            members: Vec::new(),
        });

        let mut placeholders = HashMap::new();
        placeholders.insert("channel".to_string(), format!("{}{}", channel.color, channel.display_name));
        placeholders.insert("player".to_string(), player_name.to_string());
        placeholders.insert("message".to_string(), message.to_string());
        placeholders.insert("group".to_string(), group.to_string());
        placeholders.insert("prefix".to_string(), prefix.to_string());

        let formatted = replace_placeholders(&channel.format, &placeholders);
        
        // Process hex colors
        if self.config.hex_colors {
            crate::utils::text::process_hex_colors(&formatted)
        } else {
            formatted
        }
    }

    pub fn get_tab_header(&self, online: usize, max: usize) -> String {
        let mut placeholders = HashMap::new();
        placeholders.insert("server".to_string(), "Pumpkin Server".to_string());
        placeholders.insert("online".to_string(), online.to_string());
        placeholders.insert("max".to_string(), max.to_string());
        placeholders.insert("time".to_string(), chrono::Local::now().format("%H:%M:%S").to_string());
        placeholders.insert("date".to_string(), chrono::Local::now().format("%Y-%m-%d").to_string());
        
        replace_placeholders(&self.config.tab_header, &placeholders)
    }

    pub fn get_tab_footer(&self) -> String {
        let mut placeholders = HashMap::new();
        placeholders.insert("server".to_string(), "Pumpkin Server".to_string());
        replace_placeholders(&self.config.tab_footer, &placeholders)
    }

    pub fn register_commands(&self, context: &mut Context) {
        crate::chattab::commands::register(self.clone(), context);
    }

    pub fn register_events(&self, context: &mut Context) -> Result<()> {
        let channels = self.channels.clone();
        let player_data = self.player_data.clone();
        
        context.register_event_handler(
            move |server, mut event| {
                let channels = channels.clone();
                let player_data = player_data.clone();
                let player = event.player.clone();
                let uuid = player.get_uuid();
                
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        // Set default channel on join
                        if !player_data.contains_key(&uuid) {
                            player_data.insert(uuid, PlayerChatData {
                                uuid,
                                channel: "global".to_string(),
                                last_message: chrono::Utc::now().timestamp(),
                                message_count: 0,
                            });
                        }
                        event
                    })
                })
            },
            EventPriority::Normal,
            false,
        )?;

        Ok(())
    }
}

impl Clone for ChatTabModule {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            channels: self.channels.clone(),
            player_data: self.player_data.clone(),
            config: self.config.clone(),
        }
    }
}