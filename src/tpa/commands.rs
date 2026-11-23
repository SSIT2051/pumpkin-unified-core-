use crate::tpa::TpaModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<TpaModule>, context: &mut Context) {
    let module_clone = module.clone();
    let tpa_cmd = Command::new("tpa")
        .permission("pumpkin-unified-core.tpa")
        .description("Request to teleport to a player")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /tpa <player>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let from_uuid = player.get_uuid();
            
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let to_uuid = target.get_uuid();
            
            if from_uuid == to_uuid {
                sender.send_message(&format_text("&cYou cannot teleport to yourself"));
                return Ok(());
            }
            
            module_clone.create_request(from_uuid, to_uuid, crate::tpa::TpaType::Tpa)?;
            sender.send_message(&format_text("&aTeleport request sent to {}", target.get_name()));
            target.send_message(&format_text("&e{} wants to teleport to you. Use /tpaccept or /tpdeny", player.get_name()));
            Ok(())
        }))
        .build();
    context.register_command(tpa_cmd);

    let module_clone = module.clone();
    let tpahere_cmd = Command::new("tpahere")
        .permission("pumpkin-unified-core.tpahere")
        .description("Request a player to teleport to you")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /tpahere <player>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let from_uuid = player.get_uuid();
            
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let to_uuid = target.get_uuid();
            
            if from_uuid == to_uuid {
                sender.send_message(&format_text("&cYou cannot teleport to yourself"));
                return Ok(());
            }
            
            module_clone.create_request(from_uuid, to_uuid, crate::tpa::TpaType::Tpahere)?;
            sender.send_message(&format_text("&aTeleport request sent to {}", target.get_name()));
            target.send_message(&format_text("&e{} wants you to teleport to them. Use /tpaccept or /tpdeny", player.get_name()));
            Ok(())
        }))
        .build();
    context.register_command(tpahere_cmd);

    let module_clone = module.clone();
    let tpaccept_cmd = Command::new("tpaccept")
        .permission("pumpkin-unified-core.tpaccept")
        .description("Accept a teleport request")
        .handler(async_handler!(move |sender, server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            let request = module_clone.get_request(uuid)
                .ok_or_else(|| anyhow::anyhow!("No pending teleport request"))?;
            
            if module_clone.is_expired(&request) {
                module_clone.remove_request(uuid);
                sender.send_message(&format_text("&cTeleport request has expired"));
                return Ok(());
            }
            
            let from_player = server.get_player_by_uuid(request.from)
                .ok_or_else(|| anyhow::anyhow!("Requester is no longer online"))?;
            
            match request.request_type {
                crate::tpa::TpaType::Tpa => {
                    let location = player.get_location();
                    from_player.teleport(location.world, location.x, location.y, location.z, location.yaw, location.pitch).await?;
                    sender.send_message(&format_text("&aAccepted teleport request from {}", from_player.get_name()));
                    from_player.send_message(&format_text("&aTeleport request accepted by {}", player.get_name()));
                }
                crate::tpa::TpaType::Tpahere => {
                    let location = from_player.get_location();
                    player.teleport(location.world, location.x, location.y, location.z, location.yaw, location.pitch).await?;
                    sender.send_message(&format_text("&aAccepted teleport request from {}", from_player.get_name()));
                    from_player.send_message(&format_text("&aTeleport request accepted by {}", player.get_name()));
                }
            }
            
            module_clone.remove_request(uuid);
            Ok(())
        }))
        .build();
    context.register_command(tpaccept_cmd);

    let module_clone = module.clone();
    let tpdeny_cmd = Command::new("tpdeny")
        .permission("pumpkin-unified-core.tpdeny")
        .description("Deny a teleport request")
        .handler(async_handler!(move |sender, server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            let request = module_clone.get_request(uuid)
                .ok_or_else(|| anyhow::anyhow!("No pending teleport request"))?;
            
            let from_player = server.get_player_by_uuid(request.from);
            if let Some(from) = from_player {
                from.send_message(&format_text("&c{} denied your teleport request", player.get_name()));
            }
            
            module_clone.remove_request(uuid);
            sender.send_message(&format_text("&aDenied teleport request"));
            Ok(())
        }))
        .build();
    context.register_command(tpdeny_cmd);

    let module_clone = module.clone();
    let tptoggle_cmd = Command::new("tptoggle")
        .permission("pumpkin-unified-core.tptoggle")
        .description("Toggle teleport requests")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            let current = module_clone.get_toggle(uuid);
            module_clone.set_toggle(uuid, !current);
            
            if !current {
                sender.send_message(&format_text("&aTeleport requests disabled"));
            } else {
                sender.send_message(&format_text("&aTeleport requests enabled"));
            }
            Ok(())
        }))
        .build();
    context.register_command(tptoggle_cmd);
}