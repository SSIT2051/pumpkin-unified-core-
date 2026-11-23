use crate::portals::PortalsModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<PortalsModule>, context: &mut Context) {
    let m = module.clone();
    let create_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.create")
        .description("Create a portal")
        .argument("create")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal create <name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let location = player.get_location();
            
            let portal = crate::portals::Portal {
                id: Uuid::new_v4(),
                name: name.to_string(),
                world: location.world.to_string(),
                x1: location.x - 1.0,
                y1: location.y,
                z1: location.z - 1.0,
                x2: location.x + 1.0,
                y2: location.y + 2.0,
                z2: location.z + 1.0,
                destination: None,
                command: None,
                particle: Some("portal".to_string()),
                material: None,
                cooldown: 5,
                permission: None,
            };
            
            m.set_portal(portal).await?;
            sender.send_message(&format_text("&aPortal {} created at your location. Use /portal setdestination to set where it leads.", name));
            Ok(())
        }))
        .build();
    context.register_command(create_cmd);

    let m = module.clone();
    let remove_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.remove")
        .description("Remove a portal")
        .argument("remove")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal remove <name>"))?;
            m.delete_portal(name).await?;
            sender.send_message(&format_text("&aPortal {} removed", name));
            Ok(())
        }))
        .build();
    context.register_command(remove_cmd);

    let m = module.clone();
    let list_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.list")
        .description("List all portals")
        .argument("list")
        .handler(async_handler!(move |sender, _server, _args| {
            let portals = m.list_portals().await?;
            if portals.is_empty() {
                sender.send_message(&format_text("&eNo portals found."));
            } else {
                for p in portals {
                    let dest = p.destination.as_ref().map(|d| format!(" -> {} ({}, {}, {})", d.world, d.x, d.y, d.z)).unwrap_or_else(|| " (no destination)".to_string());
                    let cmd = p.command.as_ref().map(|c| format!(" [cmd: {}]", c)).unwrap_or_default();
                    sender.send_message(&format_text("&a{} &7({}, {}, {}) - ({}, {}, {}){}{}", p.name, p.x1 as i32, p.y1 as i32, p.z1 as i32, p.x2 as i32, p.y2 as i32, p.z2 as i32, dest, cmd));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(list_cmd);

    let m = module.clone();
    let setdest_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.setdestination")
        .description("Set portal destination")
        .argument("setdestination")
        .argument("name")
        .argument("world")
        .argument("x")
        .argument("y")
        .argument("z")
        .argument("yaw")
        .argument("pitch")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal setdestination <name> <world> <x> <y> <z> <yaw> <pitch>"))?;
            let world = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /portal setdestination <name> <world> <x> <y> <z> <yaw> <pitch>"))?;
            let x: f64 = args.get(3).ok_or_else(|| anyhow::anyhow!("Usage: /portal setdestination <name> <world> <x> <y> <z> <yaw> <pitch>"))?.parse()?;
            let y: f64 = args.get(4).ok_or_else(|| anyhow::anyhow!("Usage: /portal setdestination <name> <world> <x> <y> <z> <yaw> <pitch>"))?.parse()?;
            let z: f64 = args.get(5).ok_or_else(|| anyhow::anyhow!("Usage: /portal setdestination <name> <world> <x> <y> <z> <yaw> <pitch>"))?.parse()?;
            let yaw: f32 = args.get(6).ok_or_else(|| anyhow::anyhow!("Usage: /portal setdestination <name> <world> <x> <y> <z> <yaw> <pitch>"))?.parse()?;
            let pitch: f32 = args.get(7).ok_or_else(|| anyhow::anyhow!("Usage: /portal setdestination <name> <world> <x> <y> <z> <yaw> <pitch>"))?.parse()?;
            
            let mut portal = m.get_portal(name).await?
                .ok_or_else(|| anyhow::anyhow!("Portal not found: {}", name))?;
            
            portal.destination = Some(crate::portals::Destination {
                world: world.to_string(),
                x,
                y,
                z,
                yaw,
                pitch,
            });
            
            m.set_portal(portal).await?;
            sender.send_message(&format_text("&aPortal {} destination set", name));
            Ok(())
        }))
        .build();
    context.register_command(setdest_cmd);

    let m = module.clone();
    let setcmd_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.setcommand")
        .description("Set portal command")
        .argument("setcommand")
        .argument("name")
        .argument("command")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal setcommand <name> <command>"))?;
            let command = args.get(2..).map(|s| s.join(" ")).unwrap_or_default();
            
            let mut portal = m.get_portal(name).await?
                .ok_or_else(|| anyhow::anyhow!("Portal not found: {}", name))?;
            
            portal.command = Some(command);
            m.set_portal(portal).await?;
            sender.send_message(&format_text("&aPortal {} command set", name));
            Ok(())
        }))
        .build();
    context.register_command(setcmd_cmd);

    let m = module.clone();
    let setparticle_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.setparticle")
        .description("Set portal particle")
        .argument("setparticle")
        .argument("name")
        .argument("particle")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal setparticle <name> <particle>"))?;
            let particle = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /portal setparticle <name> <particle>"))?;
            
            let mut portal = m.get_portal(name).await?
                .ok_or_else(|| anyhow::anyhow!("Portal not found: {}", name))?;
            
            portal.particle = Some(particle.to_string());
            m.set_portal(portal).await?;
            sender.send_message(&format_text("&aPortal {} particle set to {}", name, particle));
            Ok(())
        }))
        .build();
    context.register_command(setparticle_cmd);

    let m = module.clone();
    let setmaterial_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.setmaterial")
        .description("Set portal material")
        .argument("setmaterial")
        .argument("name")
        .argument("material")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal setmaterial <name> <material>"))?;
            let material = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /portal setmaterial <name> <material>"))?;
            
            let mut portal = m.get_portal(name).await?
                .ok_or_else(|| anyhow::anyhow!("Portal not found: {}", name))?;
            
            portal.material = Some(material.to_string());
            m.set_portal(portal).await?;
            sender.send_message(&format_text("&aPortal {} material set to {}", name, material));
            Ok(())
        }))
        .build();
    context.register_command(setmaterial_cmd);

    let m = module.clone();
    let setcooldown_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.setcooldown")
        .description("Set portal cooldown")
        .argument("setcooldown")
        .argument("name")
        .argument("cooldown")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal setcooldown <name> <seconds>"))?;
            let cooldown: i64 = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /portal setcooldown <name> <seconds>"))?.parse()?;
            
            let mut portal = m.get_portal(name).await?
                .ok_or_else(|| anyhow::anyhow!("Portal not found: {}", name))?;
            
            portal.cooldown = cooldown;
            m.set_portal(portal).await?;
            sender.send_message(&format_text("&aPortal {} cooldown set to {}s", name, cooldown));
            Ok(())
        }))
        .build();
    context.register_command(setcooldown_cmd);

    let m = module.clone();
    let setperm_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.setpermission")
        .description("Set portal permission")
        .argument("setpermission")
        .argument("name")
        .argument("permission")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal setpermission <name> <permission>"))?;
            let permission = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /portal setpermission <name> <permission>"))?;
            
            let mut portal = m.get_portal(name).await?
                .ok_or_else(|| anyhow::anyhow!("Portal not found: {}", name))?;
            
            portal.permission = Some(permission.to_string());
            m.set_portal(portal).await?;
            sender.send_message(&format_text("&aPortal {} permission set to {}", name, permission));
            Ok(())
        }))
        .build();
    context.register_command(setperm_cmd);

    let m = module.clone();
    let tp_cmd = Command::new("portal")
        .permission("pumpkin-unified-core.portal.teleport")
        .description("Teleport to a portal")
        .argument("teleport")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /portal teleport <name>"))?;
            let portal = m.get_portal(name).await?
                .ok_or_else(|| anyhow::anyhow!("Portal not found: {}", name))?;
            
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let x = (portal.x1 + portal.x2) / 2.0;
            let y = (portal.y1 + portal.y2) / 2.0;
            let z = (portal.z1 + portal.z2) / 2.0;
            
            player.teleport(portal.world.parse()?, x, y, z, 0.0, 0.0).await?;
            sender.send_message(&format_text("&aTeleported to portal: {}", name));
            Ok(())
        }))
        .build();
    context.register_command(tp_cmd);
}