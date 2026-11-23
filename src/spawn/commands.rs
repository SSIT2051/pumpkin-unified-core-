use crate::spawn::SpawnModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub fn register(module: Arc<SpawnModule>, context: &mut Context) {
    let module_clone = module.clone();
    let spawn_cmd = Command::new("spawn")
        .permission("pumpkin-unified-core.spawn")
        .description("Teleport to spawn")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            
            if let Some(spawn) = module_clone.get_spawn() {
                player.teleport(spawn.world.parse()?, spawn.x, spawn.y, spawn.z, spawn.yaw, spawn.pitch).await?;
                sender.send_message(&format_text("&aTeleported to spawn"));
            } else {
                sender.send_message(&format_text("&cSpawn not set"));
            }
            Ok(())
        }))
        .build();
    context.register_command(spawn_cmd);

    let module_clone = module.clone();
    let setspawn_cmd = Command::new("setspawn")
        .permission("pumpkin-unified-core.setspawn")
        .description("Set the spawn point")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let location = player.get_location();
            
            let spawn = crate::spawn::SpawnData {
                world: location.world.to_string(),
                x: location.x,
                y: location.y,
                z: location.z,
                yaw: location.yaw,
                pitch: location.pitch,
            };
            
            module_clone.set_spawn(spawn).await?;
            sender.send_message(&format_text("&aSpawn point set"));
            Ok(())
        }))
        .build();
    context.register_command(setspawn_cmd);
}