use crate::utils::macros::*;
use pumpkin_plugin_api::{context::Context, event::EventHandler};
use std::sync::Arc;

pub struct EventModule {
    economy: Arc<crate::economy::EconomyModule>,
    warps: Arc<crate::warps::WarpsModule>,
    homes: Arc<crate::homes::HomesModule>,
    tpa: Arc<crate::tpa::TpaModule>,
    kits: Arc<crate::kits::KitsModule>,
    moderation: Arc<crate::moderation::ModerationModule>,
    gamemode: Arc<crate::gamemode::GamemodeModule>,
    permissions: Arc<crate::permissions::PermissionsModule>,
    spawn: Arc<crate::spawn::SpawnModule>,
    utility: Arc<crate::utility::UtilityModule>,
    chat: Arc<crate::chat::ChatModule>,
}

impl EventModule {
    pub fn new(
        economy: Arc<crate::economy::EconomyModule>,
        warps: Arc<crate::warps::WarpsModule>,
        homes: Arc<crate::homes::HomesModule>,
        tpa: Arc<crate::tpa::TpaModule>,
        kits: Arc<crate::kits::KitsModule>,
        moderation: Arc<crate::moderation::ModerationModule>,
        gamemode: Arc<crate::gamemode::GamemodeModule>,
        permissions: Arc<crate::permissions::PermissionsModule>,
        spawn: Arc<crate::spawn::SpawnModule>,
        utility: Arc<crate::utility::UtilityModule>,
        chat: Arc<crate::chat::ChatModule>,
    ) -> Self {
        Self {
            economy,
            warps,
            homes,
            tpa,
            kits,
            moderation,
            gamemode,
            permissions,
            spawn,
            utility,
            chat,
        }
    }

    pub fn register_handlers(&self, context: &mut Context) {
        // Register event handlers
        // Note: Event API may vary by pumpkin-plugin-api version
        // This is a placeholder that will need to be updated based on actual API
        let _module = self.clone();
        // context.register_event_handler(...);
    }
}

impl Clone for EventModule {
    fn clone(&self) -> Self {
        Self {
            economy: self.economy.clone(),
            warps: self.warps.clone(),
            homes: self.homes.clone(),
            tpa: self.tpa.clone(),
            kits: self.kits.clone(),
            moderation: self.moderation.clone(),
            gamemode: self.gamemode.clone(),
            permissions: self.permissions.clone(),
            spawn: self.spawn.clone(),
            utility: self.utility.clone(),
            chat: self.chat.clone(),
        }
    }
}
