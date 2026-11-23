use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub struct ChatModule {
    config: Arc<crate::config::ChatConfig>,
    permissions: Option<Arc<crate::permissions::PermissionsModule>>,
}

impl ChatModule {
    pub fn new() -> Self {
        let config = Arc::new(crate::config::Config::default().chat);
        Self {
            config,
            permissions: None,
        }
    }

    pub fn with_permissions(
        mut self,
        permissions: Arc<crate::permissions::PermissionsModule>,
    ) -> Self {
        self.permissions = Some(permissions);
        self
    }

    pub fn format_chat(
        &self,
        player: &pumpkin_plugin_api::player::Player,
        message: &str,
    ) -> String {
        let group = "default";
        let prefix = "";
        let suffix = "";

        if let Some(perms) = &self.permissions {
            // Try to get prefix/suffix from LuckPerms
            let _contexts = std::collections::HashMap::new();
            // Note: This would need to query the permissions module for the player's prefix/suffix
            // For now, using empty values as placeholder
        }

        self.config
            .format
            .replace("{player}", player.get_name())
            .replace("{message}", message)
            .replace("{group}", group)
            .replace("{prefix}", prefix)
            .replace("{suffix}", suffix)
    }

    pub fn register_commands(&self, context: &mut Context) {
        crate::chat::commands::register(Arc::new(self.clone()), context);
    }
}

impl Clone for ChatModule {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            permissions: self.permissions.clone(),
        }
    }
}
