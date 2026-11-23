use crate::permissions::PermissionsModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<PermissionsModule>, context: &mut Context) {
    let lp_cmd = Command::new("lp")
        .alias("luckperms")
        .permission("pumpkin-unified-core.lp.user.info")
        .description("LuckPerms commands")
        .subcommand(
            Command::new("user")
                .permission("pumpkin-unified-core.lp.user.info")
                .subcommand(
                    Command::new("info")
                        .argument("user")
                        .handler(async_handler!(move |sender, server, args| {
                            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> info"))?;
                            let uuid = server.get_player(target_name).map(|p| p.get_uuid())
                                .or_else(|| Uuid::parse_str(target_name).ok())
                                .ok_or_else(|| anyhow::anyhow!("User not found: {}", target_name))?;
                            
                            if let Some(user) = module.get_user(uuid) {
                                sender.send_message(&format_text("&6User: {} ({})", user.name, uuid));
                                sender.send_message(&format_text("&eGroups: {}", user.groups.join(", ")));
                                for perm in &user.permissions {
                                    sender.send_message(&format_text("&7  {} = {} {}", perm.node, perm.value, 
                                        if perm.context.is_empty() { String::new() } else { format!(" [{}]", serde_json::to_string(&perm.context).unwrap()) }));
                                }
                            } else {
                                sender.send_message(&format_text("&cUser not found in database"));
                            }
                            Ok(())
                        }))
                        .build()
                )
                .subcommand(
                    Command::new("permission")
                        .permission("pumpkin-unified-core.lp.user.permission")
                        .subcommand(
                            Command::new("set")
                                .argument("user")
                                .argument("permission")
                                .argument("value")
                                .handler(async_handler!(move |sender, server, args| {
                                    let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> permission set <permission> <true|false>"))?;
                                    let permission = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> permission set <permission> <true|false>"))?;
                                    let value_str = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> permission set <permission> <true|false>"))?;
                                    let value: bool = value_str.parse()?;
                                    
                                    let uuid = server.get_player(target_name).map(|p| p.get_uuid())
                                        .or_else(|| Uuid::parse_str(target_name).ok())
                                        .ok_or_else(|| anyhow::anyhow!("User not found: {}", target_name))?;
                                    
                                    module.set_user_permission(uuid, permission.to_string(), value, HashMap::new()).await?;
                                    sender.send_message(&format_text("&aSet permission {} = {} for {}", permission, value, target_name));
                                    Ok(())
                                }))
                                .build()
                        )
                        .subcommand(
                            Command::new("unset")
                                .argument("user")
                                .argument("permission")
                                .handler(async_handler!(move |sender, server, args| {
                                    let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> permission unset <permission>"))?;
                                    let permission = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> permission unset <permission>"))?;
                                    
                                    let uuid = server.get_player(target_name).map(|p| p.get_uuid())
                                        .or_else(|| Uuid::parse_str(target_name).ok())
                                        .ok_or_else(|| anyhow::anyhow!("User not found: {}", target_name))?;
                                    
                                    let mut user = module.get_user(uuid).unwrap_or_else(|| crate::permissions::UserData {
                                        uuid,
                                        name: String::new(),
                                        permissions: vec![],
                                        groups: vec![],
                                        metadata: HashMap::new(),
                                    });
                                    user.permissions.retain(|p| p.node != permission);
                                    let data = crate::storage::SerializedData::new(user.clone()).serialize()?;
                                    module.storage.set("users", &uuid.to_string(), &data).await?;
                                    module.users.insert(uuid, user);
                                    
                                    sender.send_message(&format_text("&aUnset permission {} for {}", permission, target_name));
                                    Ok(())
                                }))
                                .build()
                        )
                        .build()
                )
                .subcommand(
                    Command::new("parent")
                        .permission("pumpkin-unified-core.lp.user.parent")
                        .subcommand(
                            Command::new("add")
                                .argument("user")
                                .argument("group")
                                .handler(async_handler!(move |sender, server, args| {
                                    let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> parent add <group>"))?;
                                    let group = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /lp user <user> parent add <group>"))?;
                                    
                                    let uuid = server.get_player(target_name).map(|p| p.get_uuid())
                                        .or_else(|| Uuid::parse_str(target_name).ok())
                                        .ok_or_else(|| anyhow::anyhow!("User not found: {}", target_name))?;
                                    
                                    module.add_user_group(uuid, group.to_string()).await?;
                                    sender.send_message(&format_text("&aAdded group {} to {}", group, target_name));
                                    Ok(())
                                }))
                                .build()
                        )
                        .build()
                )
                .build()
        )
        .subcommand(
            Command::new("group")
                .permission("pumpkin-unified-core.lp.group.info")
                .subcommand(
                    Command::new("list")
                        .handler(async_handler!(move |sender, _server, _args| {
                            sender.send_message(&format_text("&6Groups:"));
                            for group in module.groups.iter() {
                                sender.send_message(&format_text("&e- {} &7(weight: {})", group.name, group.weight));
                            }
                            Ok(())
                        }))
                        .build()
                )
                .build()
        )
        .subcommand(
            Command::new("track")
                .permission("pumpkin-unified-core.lp.track.listtracks")
                .subcommand(
                    Command::new("list")
                        .handler(async_handler!(move |sender, _server, _args| {
                            sender.send_message(&format_text("&6Tracks:"));
                            for track in module.tracks.iter() {
                                sender.send_message(&format_text("&e- {}: {}", track.name, track.groups.join(" > ")));
                            }
                            Ok(())
                        }))
                        .build()
                )
                .build()
        )
        .subcommand(
            Command::new("reload")
                .permission("pumpkin-unified-core.lp.reloadconfig")
                .handler(async_handler!(move |sender, _server, _args| {
                    sender.send_message(&format_text("&aReloading configuration..."));
                    Ok(())
                }))
                .build()
        )
        .build();
    context.register_command(lp_cmd);
}