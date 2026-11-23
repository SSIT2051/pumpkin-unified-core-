use crate::chattab::ChatTabModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<ChatTabModule>, context: &mut Context) {
    let module_clone = module.clone();
    let channel_cmd = Command::new("channel")
        .alias("ch")
        .permission("pumpkin-unified-core.chattab.channel")
        .description("Manage chat channels")
        .subcommand(
            Command::new("create")
                .permission("pumpkin-unified-core.chattab.channel.create")
                .argument("name")
                .argument("display_name")
                .handler(async_handler!(move |sender, _server, args| {
                    if args.len() < 2 {
                        sender.send_message(&format_text("&cUsage: /channel create <name> <display_name>"));
                        return Ok(());
                    }
                    let name = args[0].to_lowercase();
                    let display_name = args[1..].join(" ");
                    
                    if module_clone.channels.contains_key(&name) {
                        sender.send_message(&format_text("&cChannel already exists: {}", name));
                        return Ok(());
                    }
                    
                    let channel = crate::chattab::ChatChannel {
                        id: Uuid::new_v4(),
                        name: name.clone(),
                        display_name,
                        color: "&f".to_string(),
                        format: "{channel} {player}: {message}".to_string(),
                        permission: None,
                        members: Vec::new(),
                    };
                    
                    module_clone.channels.insert(name.clone(), channel.clone());
                    module_clone.save_channel(&channel).await?;
                    
                    sender.send_message(&format_text("&aChannel created: {}", name));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("delete")
                .permission("pumpkin-unified-core.chattab.channel.delete")
                .argument("name")
                .handler(async_handler!(move |sender, _server, args| {
                    let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /channel delete <name>"))?.to_lowercase();
                    
                    if name == "global" || name == "local" || name == "staff" {
                        sender.send_message(&format_text("&cCannot delete default channel"));
                        return Ok(());
                    }
                    
                    module_clone.channels.remove(&name);
                    module_clone.storage.delete("chattab_channels", &name).await?;
                    
                    sender.send_message(&format_text("&aChannel deleted: {}", name));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("join")
                .permission("pumpkin-unified-core.chattab.channel.join")
                .argument("name")
                .handler(async_handler!(move |sender, _server, args| {
                    let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /channel join <name>"))?.to_lowercase();
                    let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
                    
                    if !module_clone.channels.contains_key(&name) {
                        sender.send_message(&format_text("&cChannel not found: {}", name));
                        return Ok(());
                    }
                    
                    let channel = module_clone.channels.get(&name).unwrap();
                    if let Some(perm) = &channel.permission {
                        if !player.has_permission(&_server, perm) {
                            sender.send_message(&format_text("&cYou don't have permission to join this channel"));
                            return Ok(());
                        }
                    }
                    
                    module_clone.set_player_channel(player.get_uuid(), &name);
                    sender.send_message(&format_text("&aJoined channel: {}", name));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("leave")
                .permission("pumpkin-unified-core.chattab.channel.leave")
                .handler(async_handler!(move |sender, _server, _args| {
                    let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
                    module_clone.set_player_channel(player.get_uuid(), "global");
                    sender.send_message(&format_text("&aLeft channel, rejoined global"));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("list")
                .permission("pumpkin-unified-core.chattab.channel.list")
                .handler(async_handler!(move |sender, _server, _args| {
                    let channels = module_clone.list_channels();
                    if channels.is_empty() {
                        sender.send_message(&format_text("&cNo channels found"));
                    } else {
                        sender.send_message(&format_text("&6&l--- Chat Channels ---"));
                        for ch in channels {
                            let perm = ch.permission.unwrap_or_else(|| "none".to_string());
                            sender.send_message(&format_text("&e{} &7({}) &8- Perm: {}", ch.display_name, ch.name, perm));
                        }
                    }
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("setcolor")
                .permission("pumpkin-unified-core.chattab.channel.setcolor")
                .argument("name")
                .argument("color")
                .handler(async_handler!(move |sender, _server, args| {
                    let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /channel setcolor <name> <color>"))?.to_lowercase();
                    let color = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /channel setcolor <name> <color>"))?;
                    
                    if let Some(mut ch) = module_clone.channels.get_mut(&name) {
                        ch.color = color.to_string();
                        module_clone.save_channel(&ch).await?;
                        sender.send_message(&format_text("&aChannel color updated: {}", name));
                    } else {
                        sender.send_message(&format_text("&cChannel not found: {}", name));
                    }
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("setformat")
                .permission("pumpkin-unified-core.chattab.channel.setformat")
                .argument("name")
                .argument("format")
                .handler(async_handler!(move |sender, _server, args| {
                    let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /channel setformat <name> <format>"))?.to_lowercase();
                    let format = args.get(1..).map(|s| s.join(" ")).unwrap_or_default();
                    
                    if let Some(mut ch) = module_clone.channels.get_mut(&name) {
                        ch.format = format;
                        module_clone.save_channel(&ch).await?;
                        sender.send_message(&format_text("&aChannel format updated: {}", name));
                    } else {
                        sender.send_message(&format_text("&cChannel not found: {}", name));
                    }
                    Ok(())
                }))
                .build()
        )
        .build();
    context.register_command(channel_cmd);

    let module_clone = module.clone();
    let chattab_cmd = Command::new("chattab")
        .permission("pumpkin-unified-core.chattab")
        .description("ChatTab management")
        .subcommand(
            Command::new("reload")
                .permission("pumpkin-unified-core.chattab.reload")
                .handler(async_handler!(move |sender, _server, _args| {
                    module_clone.initialize().await?;
                    sender.send_message(&format_text("&aChatTab reloaded"));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("tabheader")
                .permission("pumpkin-unified-core.chattab.tabheader")
                .argument("text")
                .handler(async_handler!(move |sender, _server, args| {
                    let text = args.get(0..).map(|s| s.join(" ")).unwrap_or_default();
                    module_clone.config.tab_header = text;
                    sender.send_message(&format_text("&aTab header updated"));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("tabfooter")
                .permission("pumpkin-unified-core.chattab.tabfooter")
                .argument("text")
                .handler(async_handler!(move |sender, _server, args| {
                    let text = args.get(0..).map(|s| s.join(" ")).unwrap_or_default();
                    module_clone.config.tab_footer = text;
                    sender.send_message(&format_text("&aTab footer updated"));
                    Ok(())
                }))
                .build()
        )
        .build();
    context.register_command(chattab_cmd);
}