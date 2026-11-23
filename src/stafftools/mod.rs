use crate::moderation::ModerationModule;
use crate::storage::StorageBackend;
use crate::utils::text::format_text;
use anyhow::Result;
use dashmap::DashMap;
use pumpkin_plugin_api::{
    context::Context,
    event::{AsyncPlayerChatEvent, EventHandler, EventPriority, PlayerJoinEvent, PlayerLeaveEvent},
    server::Server,
    text::TextComponent,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningEntry {
    pub id: Uuid,
    pub target: Uuid,
    pub target_name: String,
    pub issuer: Uuid,
    pub issuer_name: String,
    pub reason: String,
    pub timestamp: i64,
}

pub struct StaffToolsModule {
    storage: Arc<StorageBackend>,
    moderation: Arc<ModerationModule>,
    warnings: Arc<DashMap<Uuid, WarningEntry>>,
    vanished: Arc<DashMap<Uuid, ()>>,
    social_spy: Arc<DashMap<Uuid, Vec<Uuid>>>,
}

impl Clone for StaffToolsModule {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            moderation: self.moderation.clone(),
            warnings: self.warnings.clone(),
            vanished: self.vanished.clone(),
            social_spy: self.social_spy.clone(),
        }
    }
}

impl StaffToolsModule {
    pub fn new(storage: Arc<StorageBackend>, moderation: Arc<ModerationModule>) -> Self {
        Self {
            storage,
            moderation,
            warnings: Arc::new(DashMap::new()),
            vanished: Arc::new(DashMap::new()),
            social_spy: Arc::new(DashMap::new()),
        }
    }

    pub async fn load_data(&self) -> Result<()> {
        for key in self.storage.keys("warnings", "%").await? {
            if let Some(data) = self.storage.get("warnings", &key).await? {
                let entry: crate::storage::SerializedData<WarningEntry> =
                    crate::storage::SerializedData::deserialize(&data)?;
                self.warnings.insert(entry.data.id, entry.data);
            }
        }
        Ok(())
    }

    pub fn is_vanished(&self, uuid: Uuid) -> bool {
        self.vanished.contains_key(&uuid)
    }

    pub fn set_vanished(&self, uuid: Uuid, vanished: bool) {
        if vanished {
            self.vanished.insert(uuid, ());
        } else {
            self.vanished.remove(&uuid);
        }
    }

    pub fn register_commands(&self, context: &mut Context) {
        crate::stafftools::commands::register(Arc::new(self.clone()), context);
    }

    pub fn register_events(&self, context: &mut Context) -> Result<()> {
        let vanished = self.vanished.clone();
        context.register_event_handler(
            move |server, mut event| {
                let vanished = vanished.clone();
                let player = event.player.clone();
                let uuid = player.get_uuid();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        if vanished.contains_key(&uuid) {
                            event.join_message = TextComponent::empty();
                            for target in server.get_online_players() {
                                if target.get_uuid() != uuid {
                                    let _ = target.hide_player(player.clone()).await;
                                }
                            }
                            let _ = player.set_invisible(true).await;
                            let _ = player.set_tab_list_listed(false).await;
                        } else {
                            for target in server.get_online_players() {
                                if vanished.contains_key(&target.get_uuid()) {
                                    let _ = player.hide_player(target.clone()).await;
                                }
                            }
                        }
                        event
                    })
                })
            },
            EventPriority::Normal,
            false,
        )?;

        let vanished = self.vanished.clone();
        context.register_event_handler(
            move |_server, mut event| {
                let player = event.player.clone();
                if vanished.contains_key(&player.get_uuid()) {
                    event.leave_message = TextComponent::empty();
                }
                event
            },
            EventPriority::Normal,
            false,
        )?;

        let social_spy = self.social_spy.clone();
        context.register_event_handler(
            move |server, event| {
                let social_spy = social_spy.clone();
                let player = event.player.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        if let Some(spies) = social_spy.get(&player.get_uuid()) {
                            let message = format_text(&format!(
                                "&7[SocialSpy] &f{}&7: &r{}",
                                player.get_name(),
                                event.message
                            ));
                            for spy in spies.iter() {
                                if let Some(target) = server.get_player_by_uuid(*spy) {
                                    let _ = target.send_message(&message).await;
                                }
                            }
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
