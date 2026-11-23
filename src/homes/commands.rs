use crate::homes::HomesModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<HomesModule>, context: &mut Context) {
    let module_clone = module.clone();
    let sethome_cmd = Command::new("sethome")
        .permission("pumpkin-unified-core.sethome")
        .description("Set a home")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /sethome <name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            let homes = module_clone.list_homes(uuid).await?;
            if homes.len() >= module_clone.config.max_homes_default {
                sender.send_message(&format_text("&cYou have reached the maximum number of homes ({})", module_clone.config.max_homes_default));
                return Ok(());
            }
            
            let location = player.get_location();
            let home = crate::homes::Home {
                name: name.to_string(),
                world: location.world.to_string(),
                x: location.x,
                y: location.y,
                z: location.z,
                yaw: location.yaw,
                pitch: location.pitch,
                created_at: chrono::Utc::now().timestamp(),
            };
            
            module_clone.set_home(uuid, home).await?;
            sender.send_message(&format_text("&aHome {} set", name));
            Ok(())
        }))
        .build();
    context.register_command(sethome_cmd);

    let module_clone = module.clone();
    let home_cmd = Command::new("home")
        .permission("pumpkin-unified-core.home")
        .description("Teleport to a home")
        .argument("name")
        .handler(async_handler!(move |sender, server, args| {
            let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /home <name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            let home = module_clone.get_home(uuid, name).await?
                .ok_or_else(|| anyhow::anyhow!("Home not found: {}", name))?;
            
            player.teleport(home.world.parse()?, home.x, home.y, home.z, home.yaw, home.pitch).await?;
            sender.send_message(&format_text("&aTeleported to home: {}", name));
            Ok(())
        }))
        .build();
    context.register_command(home_cmd);

    let module_clone = module.clone();
    let delhome_cmd = Command::new("delhome")
        .permission("pumpkin-unified-core.delhome")
        .description("Delete a home")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /delhome <name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            module_clone.delete_home(uuid, name).await?;
            sender.send_message(&format_text("&aHome {} deleted", name));
            Ok(())
        }))
        .build();
    context.register_command(delhome_cmd);

    let module_clone = module.clone();
    let homes_cmd = Command::new("homes")
        .permission("pumpkin-unified-core.homes")
        .description("List your homes")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let uuid = player.get_uuid();
            
            let homes = module_clone.list_homes(uuid).await?;
            if homes.is_empty() {
                sender.send_message(&format_text("&cYou have no homes set"));
            } else {
                sender.send_message(&format_text("&6&l--- Your Homes ---"));
                for home in homes {
                    sender.send_message(&format_text("&e- {} &7({}, {}, {})", home.name, home.x as i32, home.y as i32, home.z as i32));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(homes_cmd);
}