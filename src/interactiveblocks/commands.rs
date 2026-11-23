use crate::interactiveblocks::InteractiveBlocksModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<InteractiveBlocksModule>, context: &mut Context) {
    // Elevator commands
    let m = module.clone();
    let elevator_create_cmd = Command::new("elevator")
        .permission("pumpkin-unified-core.interactive.elevator.create")
        .description("Create an elevator")
        .argument("create")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /elevator create <name>"))?;
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let location = player.get_location();
            
            let elevator = crate::interactiveblocks::Elevator {
                id: Uuid::new_v4(),
                name: name.to_string(),
                material: "white_wool".to_string(),
                floors: vec![crate::interactiveblocks::ElevatorFloor {
                    name: "floor_1".to_string(),
                    x: location.x,
                    y: location.y,
                    z: location.z,
                }],
            };
            
            m.save_elevator(elevator).await?;
            sender.send_message(&format_text("&aElevator {} created at your location. Use /elevator setfloor to add more floors.", name));
            Ok(())
        }))
        .build();
    context.register_command(elevator_create_cmd);

    let m = module.clone();
    let elevator_remove_cmd = Command::new("elevator")
        .permission("pumpkin-unified-core.interactive.elevator.remove")
        .description("Remove an elevator")
        .argument("remove")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /elevator remove <name>"))?;
            m.remove_elevator(name).await?;
            sender.send_message(&format_text("&aElevator {} removed", name));
            Ok(())
        }))
        .build();
    context.register_command(elevator_remove_cmd);

    let m = module.clone();
    let elevator_list_cmd = Command::new("elevator")
        .permission("pumpkin-unified-core.interactive.elevator.list")
        .description("List all elevators")
        .argument("list")
        .handler(async_handler!(move |sender, _server, _args| {
            let elevators = m.list_elevators().await;
            if elevators.is_empty() {
                sender.send_message(&format_text("&cNo elevators found"));
            } else {
                sender.send_message(&format_text("&6&l--- Elevators ---"));
                for e in elevators {
                    sender.send_message(&format_text("&e{} &7({} floors, material: {})", e.name, e.floors.len(), e.material));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(elevator_list_cmd);

    let m = module.clone();
    let elevator_setfloor_cmd = Command::new("elevator")
        .permission("pumpkin-unified-core.interactive.elevator.setfloor")
        .description("Set elevator floor")
        .argument("setfloor")
        .argument("name")
        .argument("floor")
        .argument("x")
        .argument("y")
        .argument("z")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /elevator setfloor <name> <floor> <x> <y> <z>"))?;
            let floor_name = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /elevator setfloor <name> <floor> <x> <y> <z>"))?;
            let x: f64 = args.get(3).ok_or_else(|| anyhow::anyhow!("Usage: /elevator setfloor <name> <floor> <x> <y> <z>"))?.parse()?;
            let y: f64 = args.get(4).ok_or_else(|| anyhow::anyhow!("Usage: /elevator setfloor <name> <floor> <x> <y> <z>"))?.parse()?;
            let z: f64 = args.get(5).ok_or_else(|| anyhow::anyhow!("Usage: /elevator setfloor <name> <floor> <x> <y> <z>"))?.parse()?;
            
            let mut elevator = match m.elevators.get(name) {
                Some(e) => e.clone(),
                None => {
                    sender.send_message(&format_text("&cElevator not found: {}", name));
                    return Ok(());
                }
            };
            
            elevator.floors.push(crate::interactiveblocks::ElevatorFloor {
                name: floor_name.to_string(),
                x,
                y,
                z,
            });
            
            m.save_elevator(elevator).await?;
            sender.send_message(&format_text("&aFloor {} added to elevator {}", floor_name, name));
            Ok(())
        }))
        .build();
    context.register_command(elevator_setfloor_cmd);

    let m = module.clone();
    let elevator_setmaterial_cmd = Command::new("elevator")
        .permission("pumpkin-unified-core.interactive.elevator.setmaterial")
        .description("Set elevator material")
        .argument("setmaterial")
        .argument("name")
        .argument("material")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /elevator setmaterial <name> <material>"))?;
            let material = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /elevator setmaterial <name> <material>"))?;
            
            let mut elevator = match m.elevators.get(name) {
                Some(e) => e.clone(),
                None => {
                    sender.send_message(&format_text("&cElevator not found: {}", name));
                    return Ok(());
                }
            };
            
            elevator.material = material.to_string();
            m.save_elevator(elevator).await?;
            sender.send_message(&format_text("&aElevator {} material set to {}", name, material));
            Ok(())
        }))
        .build();
    context.register_command(elevator_setmaterial_cmd);

    // Armor Stand commands
    let m = module.clone();
    let armorstand_pose_cmd = Command::new("armorstand")
        .permission("pumpkin-unified-core.interactive.armorstand.pose")
        .description("Create or edit armor stand pose")
        .argument("pose")
        .argument("name")
        .argument("part")
        .argument("x")
        .argument("y")
        .argument("z")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /armorstand pose <name> <part> <x> <y> <z>"))?;
            let part = args.get(2).ok_or_else(|| anyhow::anyhow!("Usage: /armorstand pose <name> <part> <x> <y> <z>"))?;
            let x: f32 = args.get(3).ok_or_else(|| anyhow::anyhow!("Usage: /armorstand pose <name> <part> <x> <y> <z>"))?.parse()?;
            let y: f32 = args.get(4).ok_or_else(|| anyhow::anyhow!("Usage: /armorstand pose <name> <part> <x> <y> <z>"))?.parse()?;
            let z: f32 = args.get(5).ok_or_else(|| anyhow::anyhow!("Usage: /armorstand pose <name> <part> <x> <y> <z>"))?.parse()?;
            
            let mut pose = match m.armor_stands.get(name) {
                Some(p) => p.clone(),
                None => crate::interactiveblocks::ArmorStandPose {
                    name: name.to_string(),
                    head: (0.0, 0.0, 0.0),
                    body: (0.0, 0.0, 0.0),
                    left_arm: (0.0, 0.0, 0.0),
                    right_arm: (0.0, 0.0, 0.0),
                    left_leg: (0.0, 0.0, 0.0),
                    right_leg: (0.0, 0.0, 0.0),
                },
            };
            
            match part.to_lowercase().as_str() {
                "head" => pose.head = (x, y, z),
                "body" => pose.body = (x, y, z),
                "leftarm" => pose.left_arm = (x, y, z),
                "rightarm" => pose.right_arm = (x, y, z),
                "leftleg" => pose.left_leg = (x, y, z),
                "rightleg" => pose.right_leg = (x, y, z),
                _ => {
                    sender.send_message(&format_text("&cInvalid part. Use: head, body, leftarm, rightarm, leftleg, rightleg"));
                    return Ok(());
                }
            }
            
            m.save_armor_stand(pose).await?;
            sender.send_message(&format_text("&aArmor stand {} {} pose set", name, part));
            Ok(())
        }))
        .build();
    context.register_command(armorstand_pose_cmd);

    let m = module.clone();
    let armorstand_remove_cmd = Command::new("armorstand")
        .permission("pumpkin-unified-core.interactive.armorstand.remove")
        .description("Remove an armor stand pose")
        .argument("remove")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /armorstand remove <name>"))?;
            m.remove_armor_stand(name).await?;
            sender.send_message(&format_text("&aArmor stand {} removed", name));
            Ok(())
        }))
        .build();
    context.register_command(armorstand_remove_cmd);

    let m = module.clone();
    let armorstand_list_cmd = Command::new("armorstand")
        .permission("pumpkin-unified-core.interactive.armorstand.list")
        .description("List all armor stand poses")
        .argument("list")
        .handler(async_handler!(move |sender, _server, _args| {
            let poses = m.list_armor_stands().await;
            if poses.is_empty() {
                sender.send_message(&format_text("&cNo armor stand poses found"));
            } else {
                sender.send_message(&format_text("&6&l--- Armor Stand Poses ---"));
                for p in poses {
                    sender.send_message(&format_text("&e{}", p.name));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(armorstand_list_cmd);

    // Command Sign commands
    let m = module.clone();
    let commandsign_create_cmd = Command::new("commandsign")
        .permission("pumpkin-unified-core.interactive.commandsign.create")
        .description("Create a command sign")
        .argument("create")
        .argument("name")
        .argument("command")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /commandsign create <name> <command>"))?;
            let command = args.get(2..).map(|s| s.join(" ")).unwrap_or_default();
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            
            let sign = crate::interactiveblocks::CommandSign {
                id: Uuid::new_v4(),
                name: name.to_string(),
                command,
            };
            
            m.save_command_sign(sign).await?;
            sender.send_message(&format_text("&aCommand sign {} created. Right-click a sign to bind it.", name));
            Ok(())
        }))
        .build();
    context.register_command(commandsign_create_cmd);

    let m = module.clone();
    let commandsign_remove_cmd = Command::new("commandsign")
        .permission("pumpkin-unified-core.interactive.commandsign.remove")
        .description("Remove a command sign")
        .argument("remove")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            let name = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /commandsign remove <name>"))?;
            m.remove_command_sign(name).await?;
            sender.send_message(&format_text("&aCommand sign {} removed", name));
            Ok(())
        }))
        .build();
    context.register_command(commandsign_remove_cmd);

    let m = module.clone();
    let commandsign_list_cmd = Command::new("commandsign")
        .permission("pumpkin-unified-core.interactive.commandsign.list")
        .description("List all command signs")
        .argument("list")
        .handler(async_handler!(move |sender, _server, _args| {
            let signs = m.list_command_signs().await;
            if signs.is_empty() {
                sender.send_message(&format_text("&cNo command signs found"));
            } else {
                sender.send_message(&format_text("&6&l--- Command Signs ---"));
                for s in signs {
                    sender.send_message(&format_text("&e{} &7- {}", s.name, s.command));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(commandsign_list_cmd);
}