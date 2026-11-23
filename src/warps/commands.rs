use crate::warps::WarpsModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<WarpsModule>, context: &mut Context) {
    let module_clone = module.clone();
    let warp_cmd = Command::new("warp")
        .permission("pumpkin-unified-core.warp")
        .description("Teleport to a warp")
        .argument("warp")
        .handler(async_handler!(move |sender, server, args| {
            let warp_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /warp <name>"))?;
            let warp = module_clone.get_warp(warp_name).await?
                .ok_or_else(|| anyhow::anyhow!("Warp not found: {}", warp_name))?;
            
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            player.teleport(warp.world.parse()?, warp.x, warp.y, warp.z, warp.yaw, warp.pitch).await?;
            sender.send_message(&format_text("&aTeleported to warp: {}", warp.name));
            Ok(())
        }))
        .build();
    context.register_command(warp_cmd);

    let module_clone = module.clone();
    let setwarp_cmd = Command::new("setwarp")
        .permission("pumpkin-unified-core.setwarp")
        .description("Create a warp at your location")
        .argument("warp")
        .handler(async_handler!(move |sender, _server, args| {
            let warp_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /setwarp <name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            
            let location = player.get_location();
            let warp = crate::warps::Warp {
                name: warp_name.to_string(),
                world: location.world.to_string(),
                x: location.x,
                y: location.y,
                z: location.z,
                yaw: location.yaw,
                pitch: location.pitch,
                creator: player.get_uuid(),
                created_at: chrono::Utc::now().timestamp(),
            };
            
            module_clone.set_warp(warp).await?;
            sender.send_message(&format_text("&aWarp {} created", warp_name));
            Ok(())
        }))
        .build();
    context.register_command(setwarp_cmd);

    let module_clone = module.clone();
    let delwarp_cmd = Command::new("delwarp")
        .permission("pumpkin-unified-core.delwarp")
        .description("Delete a warp")
        .argument("warp")
        .handler(async_handler!(move |sender, _server, args| {
            let warp_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /delwarp <name>"))?;
            module_clone.delete_warp(warp_name).await?;
            sender.send_message(&format_text("&aWarp {} deleted", warp_name));
            Ok(())
        }))
        .build();
    context.register_command(delwarp_cmd);

    let module_clone = module.clone();
    let warps_cmd = Command::new("warps")
        .permission("pumpkin-unified-core.warps")
        .description("List all warps")
        .handler(async_handler!(move |sender, _server, _args| {
            let warps = module_clone.list_warps().await?;
            if warps.is_empty() {
                sender.send_message(&format_text("&cNo warps found"));
            } else {
                sender.send_message(&format_text("&6&l--- Warps ---"));
                for warp in warps {
                    sender.send_message(&format_text("&e- {} &7({}, {}, {})", warp.name, warp.x as i32, warp.y as i32, warp.z as i32));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(warps_cmd);
}