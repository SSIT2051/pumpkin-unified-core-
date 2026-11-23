use crate::gamemode::GamemodeModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub fn register(module: Arc<GamemodeModule>, context: &mut Context) {
    let module_clone = module.clone();
    let gamemode_cmd = Command::new("gamemode")
        .alias("gm")
        .permission("pumpkin-unified-core.gamemode")
        .description("Change gamemode")
        .argument("mode")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let mode_str = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /gamemode <mode> [player]"))?;
            let target = if args.len() > 1 {
                server.get_player(args[1]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[1]))?
            } else {
                sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
            };
            
            let gamemode = match mode_str.to_lowercase().as_str() {
                "0" | "survival" | "s" => pumpkin_plugin_api::world::Gamemode::Survival,
                "1" | "creative" | "c" => pumpkin_plugin_api::world::Gamemode::Creative,
                "2" | "adventure" | "a" => pumpkin_plugin_api::world::Gamemode::Adventure,
                "3" | "spectator" | "sp" => pumpkin_plugin_api::world::Gamemode::Spectator,
                _ => {
                    sender.send_message(&format_text("&cInvalid gamemode: {}", mode_str));
                    return Ok(());
                }
            };
            
            target.set_gamemode(gamemode).await?;
            
            let mode_name = match gamemode {
                pumpkin_plugin_api::world::Gamemode::Survival => "Survival",
                pumpkin_plugin_api::world::Gamemode::Creative => "Creative",
                pumpkin_plugin_api::world::Gamemode::Adventure => "Adventure",
                pumpkin_plugin_api::world::Gamemode::Spectator => "Spectator",
            };
            
            sender.send_message(&format_text("&aSet {}'s gamemode to {}", target.get_name(), mode_name));
            if target != sender.as_player().unwrap() {
                target.send_message(&format_text("&aYour gamemode was set to {} by {}", mode_name, sender.get_name()));
            }
            Ok(())
        }))
        .build();
    context.register_command(gamemode_cmd);

    for (alias, mode, perm) in [
        ("gmc", "creative", "pumpkin-unified-core.gamemode.creative"),
        ("gms", "survival", "pumpkin-unified-core.gamemode.survival"),
        ("gma", "adventure", "pumpkin-unified-core.gamemode.adventure"),
        ("gmsp", "spectator", "pumpkin-unified-core.gamemode.spectator"),
    ] {
        let module_clone = module.clone();
        let cmd = Command::new(alias)
            .permission(perm)
            .description(&format!("Set gamemode to {}", mode))
            .argument("player")
            .handler(async_handler!(move |sender, server, args| {
                let target = if args.is_empty() {
                    sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
                } else {
                    server.get_player(args[0]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[0]))?
                };
                
                let gamemode = match mode {
                    "creative" => pumpkin_plugin_api::world::Gamemode::Creative,
                    "survival" => pumpkin_plugin_api::world::Gamemode::Survival,
                    "adventure" => pumpkin_plugin_api::world::Gamemode::Adventure,
                    "spectator" => pumpkin_plugin_api::world::Gamemode::Spectator,
                    _ => return Ok(()),
                };
                
                target.set_gamemode(gamemode).await?;
                sender.send_message(&format_text("&aSet {}'s gamemode to {}", target.get_name(), mode));
                if target != sender.as_player().unwrap() {
                    target.send_message(&format_text("&aYour gamemode was set to {} by {}", mode, sender.get_name()));
                }
                Ok(())
            }))
            .build();
        context.register_command(cmd);
    }
}