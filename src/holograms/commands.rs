use crate::holograms::HologramsModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<HologramsModule>, context: &mut Context) {
    let module_clone = module.clone();
    let create_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.create")
        .description("Create a hologram")
        .argument("create")
        .argument("name")
        .argument("text")
        .handler(async_handler!(move |sender, server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /hologram create <name> <text>"))?;
            let text = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /hologram create <name> <text>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let location = player.get_location();
            let hologram = crate::holograms::Hologram {
                id: Uuid::new_v4(),
                name: name.to_string(),
                world: location.world.to_string(),
                x: location.x,
                y: location.y,
                z: location.z,
                lines: vec![text.to_string()],
                visibility_range: 48.0,
                page_lines: 10,
            };
            module_clone.set_hologram(hologram).await?;
            sender.send_message(&format_text("&aHologram {} created", name));
            Ok(())
        }))
        .build();
    context.register_command(create_cmd);

    let module_clone = module.clone();
    let remove_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.remove")
        .description("Remove a hologram")
        .argument("remove")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /hologram remove <name>"))?;
            module_clone.delete_hologram(name).await?;
            sender.send_message(&format_text("&aHologram {} removed", name));
            Ok(())
        }))
        .build();
    context.register_command(remove_cmd);

    let module_clone = module.clone();
    let list_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.list")
        .description("List holograms")
        .argument("list")
        .handler(async_handler!(move |sender, _server, _args| {
            let holograms = module_clone.list_holograms().await?;
            if holograms.is_empty() {
                sender.send_message(&format_text("&cNo holograms found"));
            } else {
                sender.send_message(&format_text("&6&l--- Holograms ---"));
                for h in holograms {
                    sender.send_message(&format_text("&e- {} &7({}, {}, {})", h.name, h.x as i32, h.y as i32, h.z as i32));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(list_cmd);

    let module_clone = module.clone();
    let teleport_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.teleport")
        .description("Teleport to a hologram")
        .argument("teleport")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /hologram teleport <name>"))?;
            let hologram = module_clone.get_hologram(name).await?
                .ok_or_else(|| anyhow::anyhow!("Hologram not found: {}", name))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            player.teleport(hologram.world.parse()?, hologram.x, hologram.y, hologram.z, 0.0, 0.0).await?;
            sender.send_message(&format_text("&aTeleported to hologram: {}", name));
            Ok(())
        }))
        .build();
    context.register_command(teleport_cmd);

    let module_clone = module.clone();
    let edit_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.edit")
        .description("Edit a hologram's text")
        .argument("edit")
        .argument("name")
        .argument("text")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /hologram edit <name> <text>"))?;
            let text = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /hologram edit <name> <text>"))?;
            let mut hologram = module_clone.get_hologram(name).await?
                .ok_or_else(|| anyhow::anyhow!("Hologram not found: {}", name))?;
            hologram.lines = vec![text.to_string()];
            module_clone.set_hologram(hologram).await?;
            sender.send_message(&format_text("&aHologram {} edited", name));
            Ok(())
        }))
        .build();
    context.register_command(edit_cmd);

    let module_clone = module.clone();
    let move_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.move")
        .description("Move a hologram")
        .argument("move")
        .argument("name")
        .argument("x")
        .argument("y")
        .argument("z")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /hologram move <name> <x> <y> <z>"))?;
            let x: f64 = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /hologram move <name> <x> <y> <z>"))?.parse()?;
            let y: f64 = args.get(3).ok_or_else(|| anyhow::anyhow!("Usage: /hologram move <name> <x> <y> <z>"))?.parse()?;
            let z: f64 = args.get(4).ok_or_else(|| anyhow::anyhow!("Usage: /hologram move <name> <x> <y> <z>"))?.parse()?;
            let mut hologram = module_clone.get_hologram(name).await?
                .ok_or_else(|| anyhow::anyhow!("Hologram not found: {}", name))?;
            hologram.x = x;
            hologram.y = y;
            hologram.z = z;
            module_clone.set_hologram(hologram).await?;
            sender.send_message(&format_text("&aHologram {} moved", name));
            Ok(())
        }))
        .build();
    context.register_command(move_cmd);

    let module_clone = module.clone();
    let copy_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.copy")
        .description("Copy a hologram")
        .argument("copy")
        .argument("name")
        .argument("newname")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /hologram copy <name> <newname>"))?;
            let newname = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /hologram copy <name> <newname>"))?;
            let hologram = module_clone.get_hologram(name).await?
                .ok_or_else(|| anyhow::anyhow!("Hologram not found: {}", name))?;
            let mut new_hologram = hologram.clone();
            new_hologram.id = Uuid::new_v4();
            new_hologram.name = newname.to_string();
            module_clone.set_hologram(new_hologram).await?;
            sender.send_message(&format_text("&aHologram {} copied to {}", name, newname));
            Ok(())
        }))
        .build();
    context.register_command(copy_cmd);

    let module_clone = module.clone();
    let readtext_cmd = Command::new("hologram")
        .permission("pumpkin-unified-core.hologram.readtext")
        .description("Read hologram text from file")
        .argument("readtext")
        .argument("name")
        .argument("file")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /hologram readtext <name> <file>"))?;
            let file = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /hologram readtext <name> <file>"))?;
            let mut hologram = module_clone.get_hologram(name).await?
                .ok_or_else(|| anyhow::anyhow!("Hologram not found: {}", name))?;
            let content = std::fs::read_to_string(file)?;
            hologram.lines = content.lines().map(|l| l.to_string()).collect();
            module_clone.set_hologram(hologram).await?;
            sender.send_message(&format_text("&aHologram {} text loaded from {}", name, file));
            Ok(())
        }))
        .build();
    context.register_command(readtext_cmd);
}