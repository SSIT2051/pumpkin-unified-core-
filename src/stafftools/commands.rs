use crate::stafftools::StaffToolsModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use chrono::TimeZone;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    player::Player,
    server::Server,
    text::TextComponent,
};
use std::sync::Arc;
use uuid::Uuid;

fn resolve_player<'a>(server: &'a Server, name: &str) -> Option<Player> {
    server
        .get_player(name)
        .or_else(|| server.get_offline_player(name))
}

fn resolve_uuid(server: &Server, name: &str) -> Result<Uuid, anyhow::Error> {
    resolve_player(server, name)
        .map(|player| player.get_uuid())
        .or_else(|| Uuid::parse_str(name).ok())
        .ok_or_else(|| anyhow::anyhow!("Player not found: {}", name))
}

pub fn register(module: Arc<StaffToolsModule>, context: &mut Context) {
    let module_clone = module.clone();
    let command = Command::new("vanish")
        .permission("pumpkin-unified-core.staff.vanish")
        .description("Toggle vanish mode")
        .handler(async_handler!(move |sender, server, _args| {
            let player = sender
                .as_player()
                .ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            let currently_vanished = module_clone.is_vanished(uuid);
            player.set_invisible(!currently_vanished).await?;
            module_clone.set_vanished(uuid, !currently_vanished);
            if !currently_vanished {
                for target in server.get_online_players() {
                    if target.get_uuid() != uuid {
                        let _ = target.hide_player(player.clone()).await;
                    }
                }
                let _ = player.set_tab_list_listed(false).await;
                sender
                    .send_message(&format_text("&aYou are now vanished."))
                    .await?;
            } else {
                for target in server.get_online_players() {
                    let _ = target.show_player(player.clone()).await;
                }
                let _ = player.set_tab_list_listed(true).await;
                sender
                    .send_message(&format_text("&cYou are no longer vanished."))
                    .await?;
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("invsee")
        .permission("pumpkin-unified-core.staff.invsee")
        .description("View or edit a player's inventory")
        .argument("player")
        .argument("slot")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /invsee <player> [slot]"))?;
            let target = resolve_player(&server, target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            if let Some(slot_arg) = args.get(1) {
                let slot: u8 = slot_arg
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid slot number"))?;
                let existing = target.get_inventory_item(slot);
                target.set_inventory_item(slot, None).await?;
                match existing {
                    Some(_) => {
                        sender
                            .send_message(&format_text(&format!(
                                "&aCleared slot {} for {}",
                                slot,
                                target.get_name()
                            )))
                            .await?
                    }
                    None => {
                        sender
                            .send_message(&format_text(&format!(
                                "&7Slot {} for {} is already empty",
                                slot,
                                target.get_name()
                            )))
                            .await?
                    }
                }
            } else {
                sender
                    .send_message(&format_text(&format!(
                        "&7Inventory of {} ({}):",
                        target.get_name(),
                        target.get_uuid()
                    )))
                    .await?;
                for slot in 0u8..36 {
                    if let Some(stack) = target.get_inventory_item(slot) {
                        sender
                            .send_message(&format_text(&format!(
                                "&7  Slot {}: &f{} x{}",
                                slot,
                                stack.get_registry_key(),
                                stack.get_count()
                            )))
                            .await?;
                    }
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("staffchat")
        .alias("sc")
        .permission("pumpkin-unified-core.staff.staffchat")
        .description("Send a message to the staff chat")
        .argument("message")
        .handler(async_handler!(move |sender, server, args| {
            let message = args.get(0..).map(|s| s.join(" ")).unwrap_or_default();
            let formatted =
                format_text(&format!("&7[Staff] &f{}: &r{}", sender.get_name(), message));
            for target in server.get_online_players() {
                if target.has_permission(&server, "pumpkin-unified-core.staff.staffchat") {
                    target.send_message(&formatted).await?;
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("warn")
        .permission("pumpkin-unified-core.staff.warn")
        .description("Warn a player")
        .argument("player")
        .argument("reason")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /warn <player> <reason>"))?;
            let reason = args.get(1..).map(|s| s.join(" ")).unwrap_or_default();
            let target = resolve_player(&server, target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let uuid = target.get_uuid();
            let name = target.get_name();
            let id = Uuid::new_v4();
            let entry = crate::stafftools::WarningEntry {
                id,
                target: uuid,
                target_name: name.clone(),
                issuer: sender.get_uuid(),
                issuer_name: sender.get_name(),
                reason: reason.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            let data = crate::storage::SerializedData::new(entry.clone()).serialize()?;
            module_clone
                .storage
                .set("warnings", &id.to_string(), &data)
                .await?;
            module_clone.warnings.insert(id, entry);
            sender
                .send_message(&format_text(&format!("&aWarned {}: {}", name, reason)))
                .await?;
            let _ = target
                .send_message(&format_text(&format!(
                    "&cYou were warned by {}: {}",
                    sender.get_name(),
                    reason
                )))
                .await;
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("warnings")
        .permission("pumpkin-unified-core.staff.warnings")
        .description("List a player's warnings")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /warnings <player>"))?;
            let uuid = resolve_uuid(&server, target_name)?;
            let mut warnings: Vec<crate::stafftools::WarningEntry> = module_clone
                .warnings
                .iter()
                .filter(|(_, warning)| warning.target == uuid)
                .map(|(_, warning)| warning.clone())
                .collect();
            warnings.sort_by_key(|warning| warning.timestamp);
            sender
                .send_message(&format_text(&format!(
                    "&7Warnings for {} ({}):",
                    target_name, uuid
                )))
                .await?;
            if warnings.is_empty() {
                sender.send_message(&format_text("&7None.")).await?;
            } else {
                for (index, warning) in warnings.iter().enumerate() {
                    let time = chrono::TimeZone::from_timestamp(chrono::Utc, warning.timestamp, 0)
                        .map(|datetime| datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| warning.timestamp.to_string());
                    sender
                        .send_message(&format_text(&format!(
                            "&7#{}. {} &8(by {} at {})",
                            index + 1,
                            warning.reason,
                            warning.issuer_name,
                            time
                        )))
                        .await?;
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("delwarn")
        .permission("pumpkin-unified-core.staff.delwarn")
        .description("Delete a warning from a player")
        .argument("player")
        .argument("id")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /delwarn <player> <id>"))?;
            let uuid = resolve_uuid(&server, target_name)?;
            let id: usize = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Usage: /delwarn <player> <id>"))?
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid warning id"))?;
            let mut warnings: Vec<crate::stafftools::WarningEntry> = module_clone
                .warnings
                .iter()
                .filter(|(_, warning)| warning.target == uuid)
                .map(|(_, warning)| warning.clone())
                .collect();
            warnings.sort_by_key(|warning| warning.timestamp);
            if id == 0 || id > warnings.len() {
                return Err(anyhow::anyhow!("No warning #{id} for {}", target_name));
            }
            let warning = warnings.remove(id - 1);
            module_clone
                .storage
                .delete("warnings", &warning.id.to_string())
                .await?;
            module_clone.warnings.remove(&warning.id);
            sender
                .send_message(&format_text(&format!(
                    "&aDeleted warning #{id} for {}",
                    target_name
                )))
                .await?;
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("checkban")
        .permission("pumpkin-unified-core.staff.checkban")
        .description("Check a player's ban status")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /checkban <player>"))?;
            let uuid = resolve_uuid(&server, target_name)?;
            let ban_manager = server.get_ban_manager();
            if ban_manager.is_player_banned(uuid) {
                if let Some(entry) = ban_manager.get_player_ban(uuid) {
                    let expires = entry
                        .expires
                        .as_deref()
                        .map(|expires| format!(" expires at {expires}"))
                        .unwrap_or_default();
                    sender
                        .send_message(&format_text(&format!(
                            "&c{} is banned{}.\n&7Reason: {}",
                            target_name, expires, entry.reason
                        )))
                        .await?;
                } else {
                    sender
                        .send_message(&format_text(&format!("&c{} is banned.", target_name)))
                        .await?;
                }
            } else {
                sender
                    .send_message(&format_text(&format!("&a{} is not banned.", target_name)))
                    .await?;
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("checkmute")
        .permission("pumpkin-unified-core.staff.checkmute")
        .description("Check a player's mute status")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /checkmute <player>"))?;
            let uuid = resolve_uuid(&server, target_name)?;
            if let Some(mute) = module_clone.moderation.is_muted(uuid) {
                let expires = mute
                    .expires_at
                    .map(|expires| format!(" expires at {expires}"))
                    .unwrap_or_default();
                sender
                    .send_message(&format_text(&format!(
                        "&c{} is muted{}.\n&7Reason: {}",
                        target_name, expires, mute.reason
                    )))
                    .await?;
            } else {
                sender
                    .send_message(&format_text(&format!("&a{} is not muted.", target_name)))
                    .await?;
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("freeze")
        .permission("pumpkin-unified-core.staff.freeze")
        .description("Freeze a player in place")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /freeze <player>"))?;
            let target = resolve_player(&server, target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            target.set_movement_locked(true).await?;
            sender
                .send_message(&format_text(&format!("&aFrozen {}", target.get_name())))
                .await?;
            let _ = target
                .send_message(&format_text(&format!(
                    "&cYou have been frozen by {}",
                    sender.get_name()
                )))
                .await;
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("unfreeze")
        .permission("pumpkin-unified-core.staff.unfreeze")
        .description("Unfreeze a player")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /unfreeze <player>"))?;
            let target = resolve_player(&server, target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            target.set_movement_locked(false).await?;
            sender
                .send_message(&format_text(&format!("&aUnfroze {}", target.get_name())))
                .await?;
            let _ = target
                .send_message(&format_text(&format!(
                    "&cYou have been unfrozen by {}",
                    sender.get_name()
                )))
                .await;
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("socialspy")
        .permission("pumpkin-unified-core.staff.socialspy")
        .description("Toggle social spying on a player")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /socialspy <player>"))?;
            let target = resolve_player(&server, target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            let uuid = target.get_uuid();
            let issuer = sender.get_uuid();
            let spies = module_clone.social_spy.entry(uuid).or_default();
            if spies.contains(&issuer) {
                spies.retain(|spy| *spy != issuer);
                sender
                    .send_message(&format_text(&format!(
                        "&aStopped social spying on {}",
                        target.get_name()
                    )))
                    .await?;
            } else {
                spies.push(issuer);
                sender
                    .send_message(&format_text(&format!(
                        "&aYou are now social spying on {}",
                        target.get_name()
                    )))
                    .await?;
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("clearchat")
        .permission("pumpkin-unified-core.staff.clearchat")
        .description("Clear the chat for all players")
        .handler(async_handler!(move |sender, server, _args| {
            let sender_name = sender.get_name();
            for target in server.get_online_players() {
                for _ in 0..100 {
                    target.send_message(&TextComponent::empty()).await?;
                }
                target
                    .send_message(&format_text(&format!(
                        "&7» &aChat was cleared by {sender_name}"
                    )))
                    .await?;
            }
            Ok(())
        }))
        .build();
    context.register_command(command);

    let module_clone = module.clone();
    let command = Command::new("broadcaststaff")
        .permission("pumpkin-unified-core.staff.broadcaststaff")
        .description("Send a message to all staff members")
        .argument("message")
        .handler(async_handler!(move |sender, server, args| {
            let message = args.get(0..).map(|s| s.join(" ")).unwrap_or_default();
            let formatted = format_text(&format!(
                "&7[Staff Broadcast] &f{}: &r{}",
                sender.get_name(),
                message
            ));
            for target in server.get_online_players() {
                if target.has_permission(&server, "pumpkin-unified-core.staff.broadcaststaff") {
                    target.send_message(&formatted).await?;
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(command);
}
