use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct GamemodeModule;

impl GamemodeModule {
    pub fn new() -> Self {
        Self
    }

    pub fn register_commands(&self, context: &mut Context) {
        crate::gamemode::commands::register(Arc::new(self.clone()), context);
    }
}

impl Clone for GamemodeModule {
    fn clone(&self) -> Self {
        Self
    }
}
