use crate::moderation::ModerationModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use crate::utils::time::parse_duration;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<ModerationModule>, context: &mut Context) {
    let module_clone = module.clone();
    let kick_cmd = Command::new("kick")
        .permission("pumpkin-unified-core.kick")
        .description("Kick a player")
        .argument("player")
        .argument("reason")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /kick <player> [reason]"))?;
            let reason = args.get(1..).map(|s| s.join(" ")).unwrap_or_else(|| "No reason specified".to_string());
            
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            
            target.kick(&format_text(&reason)).await?;
            sender.send_message(&format_text("&aKicked {}: {}", target.get_name(), reason));
            Ok(())
        }))
        .build();
    context.register_command(kick_cmd);

    let module_clone = module.clone();
    let ban_cmd = Command::new("ban")
        .permission("pumpkin-unified-core.ban")
        .description("Ban a player")
        .argument("player")
        .argument("reason")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /ban <player> [reason]"))?;
            let reason = args.get(1..).map(|s| s.join(" ")).unwrap_or_else(|| "No reason specified".to_string());
            
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let uuid = target.get_uuid();
            let name = target.get_name();
            
            module_clone.ban(uuid, name.clone(), reason.clone(), sender.get_uuid(), None).await?;
            target.kick(&format_text("&cYou have been banned: {}", reason)).await?;
            sender.send_message(&format_text("&aBanned {}: {}", name, reason));
            Ok(())
        }))
        .build();
    context.register_command(ban_cmd);

    let module_clone = module.clone();
    let tempban_cmd = Command::new("tempban")
        .permission("pumpkin-unified-core.tempban")
        .description("Temporarily ban a player")
        .argument("player")
        .argument("duration")
        .argument("reason")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /tempban <player> <duration> [reason]"))?;
            let duration_str = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /tempban <player> <duration> [reason]"))?;
            let reason = args.get(2..).map(|s| s.join(" ")).unwrap_or_else(|| "No reason specified".to_string());
            
            let duration = parse_duration(duration_str)?;
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let uuid = target.get_uuid();
            let name = target.get_name();
            
            module_clone.ban(uuid, name.clone(), reason.clone(), sender.get_uuid(), Some(duration.num_seconds() as u64)).await?;
            target.kick(&format_text("&cYou have been temporarily banned: {} ({})", reason, duration_str)).await?;
            sender.send_message(&format_text("&aTemporarily banned {} for {}: {}", name, duration_str, reason));
            Ok(())
        }))
        .build();
    context.register_command(tempban_cmd);

    let module_clone = module.clone();
    let unban_cmd = Command::new("unban")
        .permission("pumpkin-unified-core.unban")
        .description("Unban a player")
        .argument("player")
        .handler(async_handler!(move |sender, _server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /unban <player>"))?;
            let uuid = Uuid::parse_str(target_name).or_else(|_| {
                _server.get_offline_player(target_name).map(|p| p.get_uuid())
            }).ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            
            module_clone.unban(uuid).await?;
            sender.send_message(&format_text("&aUnbanned {}", target_name));
            Ok(())
        }))
        .build();
    context.register_command(unban_cmd);

    let module_clone = module.clone();
    let mute_cmd = Command::new("mute")
        .permission("pumpkin-unified-core.mute")
        .description("Mute a player")
        .argument("player")
        .argument("reason")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /mute <player> [reason]"))?;
            let reason = args.get(1..).map(|s| s.join(" ")).unwrap_or_else(|| "No reason specified".to_string());
            
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let uuid = target.get_uuid();
            let name = target.get_name();
            
            module_clone.mute(uuid, name.clone(), reason.clone(), sender.get_uuid(), None).await?;
            sender.send_message(&format_text("&aMuted {}: {}", name, reason));
            target.send_message(&format_text("&cYou have been muted: {}", reason));
            Ok(())
        }))
        .build();
    context.register_command(mute_cmd);

    let module_clone = module.clone();
    let unmute_cmd = Command::new("unmute")
        .permission("pumpkin-unified-core.unmute")
        .description("Unmute a player")
        .argument("player")
        .handler(async_handler!(move |sender, _server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /unmute <player>"))?;
            let uuid = Uuid::parse_str(target_name).or_else(|_| {
                _server.get_offline_player(target_name).map(|p| p.get_uuid())
            }).ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            
            module_clone.unmute(uuid).await?;
            sender.send_message(&format_text("&aUnmuted {}", target_name));
            Ok(())
        }))
        .build();
    context.register_command(unmute_cmd);

    let module_clone = module.clone();
    let jail_cmd = Command::new("jail")
        .permission("pumpkin-unified-core.jail")
        .description("Jail a player")
        .argument("player")
        .argument("reason")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /jail <player> [reason]"))?;
            let reason = args.get(1..).map(|s| s.join(" ")).unwrap_or_else(|| "No reason specified".to_string());
            
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let uuid = target.get_uuid();
            let name = target.get_name();
            
            if let Some(jail_loc) = &module_clone.config.jail_location {
                let _ = target.teleport(jail_loc.world.parse().unwrap(), jail_loc.x, jail_loc.y, jail_loc.z, jail_loc.yaw, jail_loc.pitch).await;
            }
            
            module_clone.jail(uuid, name.clone(), reason.clone(), sender.get_uuid(), None).await?;
            sender.send_message(&format_text("&aJailed {}: {}", name, reason));
            target.send_message(&format_text("&cYou have been jailed: {}", reason));
            Ok(())
        }))
        .build();
    context.register_command(jail_cmd);

    let module_clone = module.clone();
    let unjail_cmd = Command::new("unjail")
        .permission("pumpkin-unified-core.unjail")
        .description("Unjail a player")
        .argument("player")
        .handler(async_handler!(move |sender, _server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /unjail <player>"))?;
            let uuid = Uuid::parse_str(target_name).or_else(|_| {
                _server.get_offline_player(target_name).map(|p| p.get_uuid())
            }).ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            
            module_clone.unjail(uuid).await?;
            sender.send_message(&format_text("&aUnjailed {}", target_name));
            Ok(())
        }))
        .build();
    context.register_command(unjail_cmd);

    let module_clone = module.clone();
    let fly_cmd = Command::new("fly")
        .permission("pumpkin-unified-core.fly")
        .description("Toggle flight")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target = if args.is_empty() {
                sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
            } else {
                server.get_player(args[0]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[0]))?
            };
            
            let flying = target.get_allow_flight();
            target.set_allow_flight(!flying).await?;
            target.set_flying(!flying).await?;
            
            let msg = if !flying { "&aFlight enabled" } else { "&cFlight disabled" };
            sender.send_message(&format_text(msg));
            if target != sender.as_player().unwrap() {
                target.send_message(&format_text(msg));
            }
            Ok(())
        }))
        .build();
    context.register_command(fly_cmd);

    let module_clone = module.clone();
    let god_cmd = Command::new("god")
        .permission("pumpkin-unified-core.god")
        .description("Toggle god mode")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target = if args.is_empty() {
                sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
            } else {
                server.get_player(args[0]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[0]))?
            };
            
            let invulnerable = target.is_invulnerable();
            target.set_invulnerable(!invulnerable).await?;
            
            let msg = if !invulnerable { "&aGod mode enabled" } else { "&cGod mode disabled" };
            sender.send_message(&format_text(msg));
            if target != sender.as_player().unwrap() {
                target.send_message(&format_text(msg));
            }
            Ok(())
        }))
        .build();
    context.register_command(god_cmd);

    let module_clone = module.clone();
    let heal_cmd = Command::new("heal")
        .permission("pumpkin-unified-core.heal")
        .description("Heal a player")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target = if args.is_empty() {
                sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
            } else {
                server.get_player(args[0]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[0]))?
            };
            
            target.set_health(target.get_max_health()).await?;
            target.set_food_level(20).await?;
            target.set_saturation(20.0).await?;
            target.clear_fire().await?;
            
            sender.send_message(&format_text("&aHealed {}", target.get_name()));
            if target != sender.as_player().unwrap() {
                target.send_message(&format_text("&aYou have been healed by {}", sender.get_name()));
            }
            Ok(())
        }))
        .build();
    context.register_command(heal_cmd);

    let module_clone = module.clone();
    let feed_cmd = Command::new("feed")
        .permission("pumpkin-unified-core.feed")
        .description("Feed a player")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target = if args.is_empty() {
                sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
            } else {
                server.get_player(args[0]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[0]))?
            };
            
            target.set_food_level(20).await?;
            target.set_saturation(20.0).await?;
            
            sender.send_message(&format_text("&aFed {}", target.get_name()));
            if target != sender.as_player().unwrap() {
                target.send_message(&format_text("&aYou have been fed by {}", sender.get_name()));
            }
            Ok(())
        }))
        .build();
    context.register_command(feed_cmd);

    let module_clone = module.clone();
    let vanish_cmd = Command::new("vanish")
        .permission("pumpkin-unified-core.vanish")
        .description("Toggle vanish")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let vanished = player.is_invisible();
            player.set_invisible(!vanished).await?;
            
            let msg = if !vanished { "&aVanish enabled" } else { "&cVanish disabled" };
            sender.send_message(&format_text(msg));
            Ok(())
        }))
        .build();
    context.register_command(vanish_cmd);
}