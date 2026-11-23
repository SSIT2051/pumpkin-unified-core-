use crate::utility::UtilityModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<UtilityModule>, context: &mut Context) {
    let module_clone = module.clone();
    let afk_cmd = Command::new("afk")
        .permission("pumpkin-unified-core.afk")
        .description("Toggle AFK status")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target = if args.is_empty() {
                sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
            } else {
                server.get_player(args[0]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[0]))?
            };
            
            let afk = target.is_afk();
            target.set_afk(!afk).await?;
            
            let msg = if !afk { "&aYou are now AFK" } else { "&aYou are no longer AFK" };
            sender.send_message(&format_text(msg));
            if target != sender.as_player().unwrap() {
                target.send_message(&format_text(msg));
            }
            Ok(())
        }))
        .build();
    context.register_command(afk_cmd);

    let module_clone = module.clone();
    let rules_cmd = Command::new("rules")
        .permission("pumpkin-unified-core.rules")
        .description("Show server rules")
        .handler(async_handler!(move |sender, _server, _args| {
            sender.send_message(&format_text("&6&l--- Server Rules ---"));
            for (i, rule) in module_clone.config.rules.iter().enumerate() {
                sender.send_message(&format_text("&e{}. {}", i + 1, rule));
            }
            Ok(())
        }))
        .build();
    context.register_command(rules_cmd);

    let module_clone = module.clone();
    let motd_cmd = Command::new("motd")
        .permission("pumpkin-unified-core.motd")
        .description("Show message of the day")
        .handler(async_handler!(move |sender, _server, _args| {
            sender.send_message(&format_text("&6&l--- MOTD ---"));
            sender.send_message(&format_text("&e{}", module_clone.config.motd));
            Ok(())
        }))
        .build();
    context.register_command(motd_cmd);

    let module_clone = module.clone();
    let list_cmd = Command::new("list")
        .alias("who")
        .permission("pumpkin-unified-core.list")
        .description("List online players")
        .handler(async_handler!(move |sender, server, _args| {
            let players = server.get_online_players();
            sender.send_message(&format_text("&6Online players ({}/{}):", players.len(), module_clone.config.max_players));
            for player in players {
                let prefix = if player.is_op() { "&c[OP]&r " } else { "" };
                sender.send_message(&format_text("&7- {}{}", prefix, player.get_name()));
            }
            Ok(())
        }))
        .build();
    context.register_command(list_cmd);

    let module_clone = module.clone();
    let whois_cmd = Command::new("whois")
        .permission("pumpkin-unified-core.whois")
        .description("Get player info")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /whois <player>"))?;
            let target = server.get_player(target_name).ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            
            sender.send_message(&format_text("&6&l--- Player Info ---"));
            sender.send_message(&format_text("&eName: &r{}", target.get_name()));
            sender.send_message(&format_text("&eUUID: &r{}", target.get_uuid()));
            sender.send_message(&format_text("&eGamemode: &r{:?}", target.get_gamemode()));
            sender.send_message(&format_text("&eHealth: &r{}/{}", target.get_health(), target.get_max_health()));
            sender.send_message(&format_text("&eFood: &r{}", target.get_food_level()));
            sender.send_message(&format_text("&eOP: &r{}", target.is_op()));
            sender.send_message(&format_text("&eAFK: &r{}", target.is_afk()));
            Ok(())
        }))
        .build();
    context.register_command(whois_cmd);

    let module_clone = module.clone();
    let speed_cmd = Command::new("speed")
        .permission("pumpkin-unified-core.speed")
        .description("Set player speed")
        .argument("player")
        .argument("speed")
        .handler(async_handler!(move |sender, server, args| {
            let target = if args.len() == 1 {
                sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?
            } else {
                server.get_player(args[0]).ok_or_else(|| anyhow::anyhow!("Player not found: {}", args[0]))?
            };
            
            let speed_str = if args.len() == 1 { args[0] } else { args[1] };
            let speed: f32 = speed_str.parse()?;
            
            if speed < 0.0 || speed > 10.0 {
                sender.send_message(&format_text("&cSpeed must be between 0 and 10"));
                return Ok(());
            }
            
            target.set_walk_speed(speed).await?;
            target.set_fly_speed(speed).await?;
            
            sender.send_message(&format_text("&aSet {}'s speed to {}", target.get_name(), speed));
            if target != sender.as_player().unwrap() {
                target.send_message(&format_text("&aYour speed was set to {} by {}", speed, sender.get_name()));
            }
            Ok(())
        }))
        .build();
    context.register_command(speed_cmd);

    let module_clone = module.clone();
    let compass_cmd = Command::new("compass")
        .permission("pumpkin-unified-core.compass")
        .description("Get a compass pointing to spawn")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            player.get_inventory().add_item("minecraft:compass".to_string(), 1).await?;
            sender.send_message(&format_text("&aGiven a compass"));
            Ok(())
        }))
        .build();
    context.register_command(compass_cmd);

    let module_clone = module.clone();
    let top_cmd = Command::new("top")
        .permission("pumpkin-unified-core.top")
        .description("Teleport to top block")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let location = player.get_location();
            let world = player.get_world();
            let highest_y = world.get_highest_block_y(location.x as i32, location.z as i32).await?;
            player.teleport(location.world, location.x, highest_y as f64 + 1.0, location.z, location.yaw, location.pitch).await?;
            sender.send_message(&format_text("&aTeleported to top block"));
            Ok(())
        }))
        .build();
    context.register_command(top_cmd);

    let module_clone = module.clone();
    let back_cmd = Command::new("back")
        .permission("pumpkin-unified-core.back")
        .description("Teleport to previous location")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            sender.send_message(&format_text("&cBack command not fully implemented"));
            Ok(())
        }))
        .build();
    context.register_command(back_cmd);

    let module_clone = module.clone();
    let seen_cmd = Command::new("seen")
        .permission("pumpkin-unified-core.seen")
        .description("See when a player was last online")
        .argument("player")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /seen <player>"))?;
            
            if let Some(player) = server.get_player(target_name) {
                sender.send_message(&format_text("&a{} is currently online", target_name));
            } else {
                sender.send_message(&format_text("&cPlayer {} not found or never played", target_name));
            }
            Ok(())
        }))
        .build();
    context.register_command(seen_cmd);

    let module_clone = module.clone();
    let repair_cmd = Command::new("repair")
        .permission("pumpkin-unified-core.repair")
        .description("Repair held item")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let mut item = player.get_inventory().get_item_in_main_hand();
            if item.is_air() {
                sender.send_message(&format_text("&cYou are not holding an item"));
                return Ok(());
            }
            item.set_damage(0);
            player.get_inventory().set_item_in_main_hand(item).await?;
            sender.send_message(&format_text("&aItem repaired"));
            Ok(())
        }))
        .build();
    context.register_command(repair_cmd);

    let module_clone = module.clone();
    let disposal_cmd = Command::new("disposal")
        .permission("pumpkin-unified-core.disposal")
        .description("Open a disposal inventory")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            sender.send_message(&format_text("&cDisposal command not fully implemented"));
            Ok(())
        }))
        .build();
    context.register_command(disposal_cmd);

    let module_clone = module.clone();
    let workbench_cmd = Command::new("workbench")
        .alias("wb")
        .permission("pumpkin-unified-core.workbench")
        .description("Open a crafting table")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            sender.send_message(&format_text("&cWorkbench command not fully implemented"));
            Ok(())
        }))
        .build();
    context.register_command(workbench_cmd);

    let module_clone = module.clone();
    let anvil_cmd = Command::new("anvil")
        .permission("pumpkin-unified-core.anvil")
        .description("Open an anvil")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            sender.send_message(&format_text("&cAnvil command not fully implemented"));
            Ok(())
        }))
        .build();
    context.register_command(anvil_cmd);

    let module_clone = module.clone();
    let enderchest_cmd = Command::new("enderchest")
        .alias("echest")
        .permission("pumpkin-unified-core.enderchest")
        .description("Open ender chest")
        .argument("player")
        .handler(async_handler!(move |sender, _server, _args| {
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            sender.send_message(&format_text("&cEnderchest command not fully implemented"));
            Ok(())
        }))
        .build();
    context.register_command(enderchest_cmd);

    let module_clone = module.clone();
    let weather_cmd = Command::new("weather")
        .permission("pumpkin-unified-core.weather")
        .description("Change weather")
        .argument("type")
        .argument("duration")
        .handler(async_handler!(move |sender, server, args| {
            let weather_type = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /weather <clear|rain|thunder> [duration]"))?;
            let duration = args.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(6000);
            
            for world in server.get_worlds() {
                match weather_type.to_lowercase().as_str() {
                    "clear" => world.set_weather(pumpkin_plugin_api::world::Weather::Clear, duration).await?,
                    "rain" => world.set_weather(pumpkin_plugin_api::world::Weather::Rain, duration).await?,
                    "thunder" => world.set_weather(pumpkin_plugin_api::world::Weather::Thunder, duration).await?,
                    _ => {
                        sender.send_message(&format_text("&cInvalid weather type: {}", weather_type));
                        return Ok(());
                    }
                }
            }
            
            sender.send_message(&format_text("&aSet weather to {} for {} ticks", weather_type, duration));
            Ok(())
        }))
        .build();
    context.register_command(weather_cmd);

    let module_clone = module.clone();
    let time_cmd = Command::new("time")
        .permission("pumpkin-unified-core.time")
        .description("Change time")
        .argument("value")
        .handler(async_handler!(move |sender, server, args| {
            let time_str = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /time <value|day|night|set <value>>"))?;
            
            for world in server.get_worlds() {
                match time_str.to_lowercase().as_str() {
                    "day" => world.set_time(1000).await?,
                    "night" => world.set_time(13000).await?,
                    "noon" => world.set_time(6000).await?,
                    "midnight" => world.set_time(18000).await?,
                    _ => {
                        if let Ok(time) = time_str.parse::<i64>() {
                            world.set_time(time).await?;
                        } else {
                            sender.send_message(&format_text("&cInvalid time value: {}", time_str));
                            return Ok(());
                        }
                    }
                }
            }
            
            sender.send_message(&format_text("&aSet time to {}", time_str));
            Ok(())
        }))
        .build();
    context.register_command(time_cmd);

    let module_clone = module.clone();
    let butcher_cmd = Command::new("butcher")
        .permission("pumpkin-unified-core.butcher")
        .description("Kill nearby mobs")
        .argument("radius")
        .handler(async_handler!(move |sender, _server, args| {
            let radius = args.get(0).and_then(|s| s.parse::<f64>().ok()).unwrap_or(32.0);
            sender.send_message(&format_text("&cButcher command not fully implemented (radius: {})", radius));
            Ok(())
        }))
        .build();
    context.register_command(butcher_cmd);

    let module_clone = module.clone();
    let extinguish_cmd = Command::new("extinguish")
        .permission("pumpkin-unified-core.extinguish")
        .description("Extinguish fires in radius")
        .argument("radius")
        .handler(async_handler!(move |sender, _server, args| {
            let radius = args.get(0).and_then(|s| s.parse::<f64>().ok()).unwrap_or(16.0);
            sender.send_message(&format_text("&cExtinguish command not fully implemented (radius: {})", radius));
            Ok(())
        }))
        .build();
    context.register_command(extinguish_cmd);
}