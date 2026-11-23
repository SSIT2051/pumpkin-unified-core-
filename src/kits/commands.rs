use crate::kits::KitsModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<KitsModule>, context: &mut Context) {
    let module_clone = module.clone();
    let kit_cmd = Command::new("kit")
        .permission("pumpkin-unified-core.kit")
        .description("Get a kit")
        .argument("kit")
        .handler(async_handler!(move |sender, server, args| {
            let kit_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /kit <name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            let kit = module_clone.get_kit(kit_name).await
                .ok_or_else(|| anyhow::anyhow!("Kit not found: {}", kit_name))?;
            
            if let Some(perm) = &kit.permission {
                if !player.has_permission(perm) {
                    sender.send_message(&format_text("&cYou don't have permission for this kit"));
                    return Ok(());
                }
            }
            
            if let Some(remaining) = module_clone.check_cooldown(uuid, kit_name) {
                sender.send_message(&format_text("&cKit on cooldown. Wait {} seconds", remaining));
                return Ok(());
            }
            
            for item in &kit.items {
                let _ = player.get_inventory().add_item(item.item_type.clone(), item.amount).await;
            }
            
            module_clone.set_cooldown(uuid, kit_name);
            sender.send_message(&format_text("&aReceived kit: {}", kit.display_name));
            Ok(())
        }))
        .build();
    context.register_command(kit_cmd);

    let module_clone = module.clone();
    let kits_cmd = Command::new("kits")
        .permission("pumpkin-unified-core.kits")
        .description("List available kits")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let kits = module_clone.list_kits().await;
            
            if kits.is_empty() {
                sender.send_message(&format_text("&cNo kits available"));
            } else {
                sender.send_message(&format_text("&6&l--- Available Kits ---"));
                for kit in kits {
                    if let Some(perm) = &kit.permission {
                        if !player.has_permission(perm) {
                            continue;
                        }
                    }
                    let cooldown = if kit.cooldown > 0 {
                        format!(" &7(Cooldown: {}s)", kit.cooldown)
                    } else {
                        String::new()
                    };
                    sender.send_message(&format_text("&e- {} &7{}{}", kit.display_name, kit.description.unwrap_or_default(), cooldown));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(kits_cmd);

    let module_clone = module.clone();
    let createkit_cmd = Command::new("createkit")
        .permission("pumpkin-unified-core.createkit")
        .description("Create a kit from your inventory")
        .argument("name")
        .argument("display_name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /createkit <name> <display_name>"))?;
            let display_name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /createkit <name> <display_name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            
            let kit = crate::kits::Kit {
                name: name.to_string(),
                display_name: display_name.to_string(),
                items: vec![],
                cooldown: module_clone.config.default_cooldown,
                permission: None,
                description: None,
            };
            
            module_clone.set_kit(kit).await?;
            sender.send_message(&format_text("&aKit {} created", name));
            Ok(())
        }))
        .build();
    context.register_command(createkit_cmd);

    let module_clone = module.clone();
    let showkit_cmd = Command::new("showkit")
        .permission("pumpkin-unified-core.showkit")
        .description("Show kit contents")
        .argument("kit")
        .handler(async_handler!(move |sender, _server, args| {
            let kit_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /showkit <name>"))?;
            let kit = module_clone.get_kit(kit_name).await
                .ok_or_else(|| anyhow::anyhow!("Kit not found: {}", kit_name))?;
            
            sender.send_message(&format_text("&6&l--- Kit: {} ---", kit.display_name));
            for item in &kit.items {
                sender.send_message(&format_text("&e- {} x{}", item.item_type, item.amount));
            }
            if kit.cooldown > 0 {
                sender.send_message(&format_text("&7Cooldown: {}s", kit.cooldown));
            }
            Ok(())
        }))
        .build();
    context.register_command(showkit_cmd);
}