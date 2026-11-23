use crate::playertracker::PlayerTrackerModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub fn register(module: Arc<PlayerTrackerModule>, context: &mut Context) {
    let module_clone = module.clone();
    let seen_cmd = Command::new("seen")
        .permission("pumpkin-unified-core.tracker.seen")
        .description("Check when a player was last seen")
        .argument("player")
        .handler(async_handler!(move |sender, _server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /seen <player>"))?;
            if let Some(p) = _server.get_player(target_name) {
                sender.send_message(&format_text("&a{} is online.", p.get_name()));
                return Ok(());
            }
            match module_clone.find_by_name(target_name).await? {
                Some(data) => sender.send_message(&format_text(
                    "&7{} was last seen {}.",
                    data.name,
                    PlayerTrackerModule::format_ago(data.last_seen),
                )),
                None => sender.send_message(&format_text("&cPlayer not found: {}", target_name)),
            }
            Ok(())
        }))
        .build();
    context.register_command(seen_cmd);

    let module_clone = module.clone();
    let playtime_cmd = Command::new("playtime")
        .alias("ptime")
        .permission("pumpkin-unified-core.tracker.playtime")
        .description("Check playtime")
        .handler(async_handler!(move |sender, server, args| {
            let uuid = if args.is_empty() {
                sender.get_uuid()
            } else {
                match server.get_player(&args[0]) {
                    Some(p) => p.get_uuid(),
                    None => {
                        sender.send_message(&format_text("&cPlayer not found: {}", args[0]));
                        return Ok(());
                    }
                }
            };
            let playtime = module_clone.get_playtime(uuid).await?;
            let name = server
                .get_player_by_uuid(uuid)
                .map(|p| p.get_name())
                .unwrap_or_else(|| "Unknown".to_string());
            sender.send_message(&format_text(
                "&a{}'s playtime: {}",
                name,
                PlayerTrackerModule::format_duration(playtime),
            ));
            Ok(())
        }))
        .build();
    context.register_command(playtime_cmd);

    let module_clone = module.clone();
    let playtime_top_cmd = Command::new("playtimetop")
        .permission("pumpkin-unified-core.tracker.playtime.top")
        .description("Show top playtimes")
        .argument("page")
        .handler(async_handler!(move |sender, _server, args| {
            let page = args.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).max(1);
            let per_page = 10;
            
            // Get all players from storage
            let keys = module_clone.storage.keys("playertracker", "%").await?;
            let mut players = Vec::new();
            
            for key in keys {
                if let Ok(uuid) = uuid::Uuid::parse_str(&key) {
                    if let Some(data) = module_clone.get_data(uuid).await? {
                        let playtime = module_clone.get_playtime(uuid).await?;
                        players.push((data.name, playtime));
                    }
                }
            }
            
            players.sort_by(|a, b| b.1.cmp(&a.1));
            
            if players.is_empty() {
                sender.send_message(&format_text("&cNo playtime data found"));
                return Ok(());
            }
            
            let start = (page - 1) * per_page;
            let end = (start + per_page).min(players.len());
            
            if start >= players.len() {
                sender.send_message(&format_text("&cPage {} does not exist", page));
                return Ok(());
            }
            
            sender.send_message(&format_text("&6&l--- Playtime Top (Page {}) ---", page));
            for (i, (name, playtime)) in players[start..end].iter().enumerate() {
                sender.send_message(&format_text(
                    "&e{}. &a{} &7- {}",
                    start + i + 1,
                    name,
                    PlayerTrackerModule::format_duration(*playtime),
                ));
            }
            Ok(())
        }))
        .build();
    context.register_command(playtime_top_cmd);

    let module_clone = module.clone();
    let firstjoin_cmd = Command::new("firstjoin")
        .permission("pumpkin-unified-core.tracker.firstjoin")
        .description("Check a player's first join")
        .argument("player")
        .handler(async_handler!(move |sender, _server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /firstjoin <player>"))?;
            match module_clone.find_by_name(target_name).await? {
                Some(data) => {
                    let first_join = chrono::DateTime::from_timestamp(data.first_join, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    sender.send_message(&format_text(
                        "&7{} first joined on {} ({})",
                        data.name,
                        first_join,
                        PlayerTrackerModule::format_ago(data.first_join),
                    ));
                }
                None => sender.send_message(&format_text("&cPlayer not found: {}", target_name)),
            }
            Ok(())
        }))
        .build();
    context.register_command(firstjoin_cmd);

    let module_clone = module.clone();
    let iphistory_cmd = Command::new("iphistory")
        .permission("pumpkin-unified-core.tracker.iphistory")
        .description("Check a player's IP history")
        .argument("player")
        .handler(async_handler!(move |sender, _server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /iphistory <player>"))?;
            match module_clone.find_by_name(target_name).await? {
                Some(data) => {
                    let ips = module_clone.get_ip_history(data.uuid).await?;
                    if ips.is_empty() {
                        sender.send_message(&format_text("&cNo IP history for {}", data.name));
                    } else {
                        sender.send_message(&format_text("&6&l--- IP History for {} ---", data.name));
                        for ip in ips {
                            let first = chrono::DateTime::from_timestamp(ip.first_seen, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            let last = chrono::DateTime::from_timestamp(ip.last_seen, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            sender.send_message(&format_text(
                                "&e{} &7- Joins: {} &7- First: {} &7- Last: {}",
                                ip.ip, ip.join_count, first, last
                            ));
                        }
                    }
                }
                None => sender.send_message(&format_text("&cPlayer not found: {}", target_name)),
            }
            Ok(())
        }))
        .build();
    context.register_command(iphhistory_cmd);

    let module_clone = module.clone();
    let session_cmd = Command::new("session")
        .permission("pumpkin-unified-core.tracker.session")
        .description("Check a player's sessions")
        .argument("player")
        .handler(async_handler!(move |sender, _server, args| {
            let target_name = args
                .get(0)
                .ok_or_else(|| anyhow::anyhow!("Usage: /session <player>"))?;
            match module_clone.find_by_name(target_name).await? {
                Some(data) => {
                    let sessions = module_clone.get_sessions(data.uuid).await?;
                    if sessions.is_empty() {
                        sender.send_message(&format_text("&cNo sessions for {}", data.name));
                    } else {
                        sender.send_message(&format_text("&6&l--- Sessions for {} ---", data.name));
                        for s in sessions.iter().rev().take(20) {
                            let login = chrono::DateTime::from_timestamp(s.login_time, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "Unknown".to_string());
                            let logout = s.logout_time.map(|t| {
                                chrono::DateTime::from_timestamp(t, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unknown".to_string())
                            }).unwrap_or_else(|| "Online".to_string());
                            let duration = s.duration.map(|d| PlayerTrackerModule::format_duration(d)).unwrap_or_else(|| "Current".to_string());
                            sender.send_message(&format_text(
                                "&7Login: {} &7| Logout: {} &7| Duration: {} &7| IP: {}",
                                login, logout, duration, s.ip
                            ));
                        }
                    }
                }
                None => sender.send_message(&format_text("&cPlayer not found: {}", target_name)),
            }
            Ok(())
        }))
        .build();
    context.register_command(session_cmd);

    let module_clone = module.clone();
    let auditlog_cmd = Command::new("auditlog")
        .permission("pumpkin-unified-core.tracker.auditlog")
        .description("View audit log")
        .subcommand(
            Command::new("player")
                .permission("pumpkin-unified-core.tracker.auditlog")
                .argument("player")
                .handler(async_handler!(move |sender, _server, args| {
                    let target_name = args
                        .get(0)
                        .ok_or_else(|| anyhow::anyhow!("Usage: /auditlog player <player>"))?;
                    match module_clone.find_by_name(target_name).await? {
                        Some(data) => {
                            let audits = module_clone.get_audit_log(data.uuid, None).await?;
                            if audits.is_empty() {
                                sender.send_message(&format_text("&cNo audit log for {}", data.name));
                            } else {
                                sender.send_message(&format_text("&6&l--- Audit Log for {} (last 50) ---", data.name));
                                for a in audits.iter().rev().take(50) {
                                    let time = chrono::DateTime::from_timestamp(a.timestamp, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .unwrap_or_else(|| "Unknown".to_string());
                                    sender.send_message(&format_text(
                                        "&7[{}] &e{} &7- {} &7@ {} ({}, {}, {})",
                                        time, a.action, a.details, a.world, a.x, a.y, a.z
                                    ));
                                }
                            }
                        }
                        None => sender.send_message(&format_text("&cPlayer not found: {}", target_name)),
                    }
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("block")
                .permission("pumpkin-unified-core.tracker.auditlog.block")
                .argument("player")
                .argument("block")
                .handler(async_handler!(move |sender, _server, args| {
                    let target_name = args
                        .get(0)
                        .ok_or_else(|| anyhow::anyhow!("Usage: /auditlog block <player> <block>"))?;
                    let block = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /auditlog block <player> <block>"))?;
                    match module_clone.find_by_name(target_name).await? {
                        Some(data) => {
                            let audits = module_clone.get_audit_log(data.uuid, Some("block_break")).await?;
                            let filtered: Vec<_> = audits.into_iter().filter(|a| a.details.contains(block)).collect();
                            if filtered.is_empty() {
                                sender.send_message(&format_text("&cNo block break logs for {} containing {}", data.name, block));
                            } else {
                                sender.send_message(&format_text("&6&l--- Block Break Logs for {} ({}) ---", data.name, block));
                                for a in filtered.iter().rev().take(50) {
                                    let time = chrono::DateTime::from_timestamp(a.timestamp, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .unwrap_or_else(|| "Unknown".to_string());
                                    sender.send_message(&format_text(
                                        "&7[{}] &e{} &7@ {} ({}, {}, {})",
                                        time, a.details, a.world, a.x, a.y, a.z
                                    ));
                                }
                            }
                        }
                        None => sender.send_message(&format_text("&cPlayer not found: {}", target_name)),
                    }
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("item")
                .permission("pumpkin-unified-core.tracker.auditlog.item")
                .argument("player")
                .argument("item")
                .handler(async_handler!(move |sender, _server, args| {
                    let target_name = args
                        .get(0)
                        .ok_or_else(|| anyhow::anyhow!("Usage: /auditlog item <player> <item>"))?;
                    let item = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /auditlog item <player> <item>"))?;
                    match module_clone.find_by_name(target_name).await? {
                        Some(data) => {
                            let audits = module_clone.get_audit_log(data.uuid, Some("item_pickup")).await?;
                            let audits2 = module_clone.get_audit_log(data.uuid, Some("item_drop")).await?;
                            let mut all = audits;
                            all.extend(audits2);
                            let filtered: Vec<_> = all.into_iter().filter(|a| a.details.contains(item)).collect();
                            if filtered.is_empty() {
                                sender.send_message(&format_text("&cNo item logs for {} containing {}", data.name, item));
                            } else {
                                sender.send_message(&format_text("&6&l--- Item Logs for {} ({}) ---", data.name, item));
                                for a in filtered.iter().rev().take(50) {
                                    let time = chrono::DateTime::from_timestamp(a.timestamp, 0)
                                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                        .unwrap_or_else(|| "Unknown".to_string());
                                    sender.send_message(&format_text(
                                        "&7[{}] &e{} &7@ {} ({}, {}, {})",
                                        time, a.details, a.world, a.x, a.y, a.z
                                    ));
                                }
                            }
                        }
                        None => sender.send_message(&format_text("&cPlayer not found: {}", target_name)),
                    }
                    Ok(())
                }))
                .build()
        )
        .build();
    context.register_command(auditlog_cmd);
}