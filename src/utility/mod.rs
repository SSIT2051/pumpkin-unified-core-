use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub struct UtilityModule {
    config: Arc<crate::config::UtilityConfig>,
}

impl UtilityModule {
    pub fn new() -> Self {
        let config = Arc::new(crate::config::Config::default().utility);
        Self { config }
    }

    pub fn with_config(config: Arc<crate::config::UtilityConfig>) -> Self {
        Self { config }
    }

    pub fn register_commands(&self, context: &mut Context) {
        crate::utility::commands::register(Arc::new(self.clone()), context);
    }
}

impl Clone for UtilityModule {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}
