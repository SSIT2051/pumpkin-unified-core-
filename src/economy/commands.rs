use crate::economy::EconomyModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn register(module: Arc<EconomyModule>, context: &mut Context) {
    let module_clone = module.clone();
    let balance_cmd = Command::new("balance")
        .alias("bal")
        .alias("money")
        .permission("pumpkin-unified-core.balance")
        .description("Check your balance")
        .handler(async_handler!(move |sender, _server, args| {
            let target = if args.is_empty() {
                sender.get_uuid()
            } else {
                match _server.get_player(&args[0]) {
                    Some(p) => p.get_uuid(),
                    None => {
                        sender.send_message(&format_text("&cPlayer not found: {}", args[0]));
                        return Ok(());
                    }
                }
            };
            
            let balance = module_clone.get_balance(target).await?;
            let formatted = module_clone.format_money(balance);
            
            if target == sender.get_uuid() {
                sender.send_message(&format_text("&aYour balance: {}", formatted));
            } else {
                sender.send_message(&format_text("&a{}'s balance: {}", 
                    _server.get_player_by_uuid(target).map(|p| p.get_name()).unwrap_or("Unknown"),
                    formatted));
            }
            Ok(())
        }))
        .build();
    context.register_command(balance_cmd);

    let module_clone = module.clone();
    let pay_cmd = Command::new("pay")
        .permission("pumpkin-unified-core.pay")
        .description("Pay another player")
        .argument("player")
        .argument("amount")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /pay <player> <amount>"))?;
            let amount_str = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /pay <player> <amount>"))?;
            let amount: f64 = amount_str.parse()?;
            
            if amount <= 0.0 {
                sender.send_message(&format_text("&cAmount must be positive"));
                return Ok(());
            }
            
            let target = server.get_player(target_name)
                .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            
            let from_uuid = sender.get_uuid();
            let to_uuid = target.get_uuid();
            
            if from_uuid == to_uuid {
                sender.send_message(&format_text("&cYou cannot pay yourself"));
                return Ok(());
            }
            
            module_clone.pay(from_uuid, to_uuid, amount).await?;
            
            let formatted = module_clone.format_money(amount);
            sender.send_message(&format_text("&aYou paid {} {}", target.get_name(), formatted));
            target.send_message(&format_text("&aYou received {} from {}", formatted, sender.get_name()));
            Ok(())
        }))
        .build();
    context.register_command(pay_cmd);

    let module_clone = module.clone();
    let baltop_cmd = Command::new("baltop")
        .alias("balancetop")
        .permission("pumpkin-unified-core.balance")
        .description("Show top balances")
        .argument("page")
        .handler(async_handler!(move |sender, _server, args| {
            let page = args.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).max(1);
            let per_page = 10;
            let top = module_clone.get_top_balances(page * per_page).await?;
            
            sender.send_message(&format_text("&6&l--- Balance Top (Page {}) ---", page));
            for (i, (uuid, balance)) in top.iter().skip((page - 1) * per_page).take(per_page).enumerate() {
                let name = _server.get_player_by_uuid(*uuid).map(|p| p.get_name()).unwrap_or("Unknown");
                let formatted = module_clone.format_money(*balance);
                sender.send_message(&format_text("&e{}. &a{} &7- {}", (page - 1) * per_page + i + 1, name, formatted));
            }
            Ok(())
        }))
        .build();
    context.register_command(baltop_cmd);

    let module_clone = module.clone();
    let eco_cmd = Command::new("eco")
        .permission("pumpkin-unified-core.eco")
        .description("Economy admin commands")
        .subcommand(
            Command::new("give")
                .permission("pumpkin-unified-core.eco.give")
                .argument("player")
                .argument("amount")
                .handler(async_handler!(move |sender, server, args| {
                    let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /eco give <player> <amount>"))?;
                    let amount_str = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /eco give <player> <amount>"))?;
                    let amount: f64 = amount_str.parse()?;
                    
                    let target = server.get_player(target_name)
                        .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
                    
                    module_clone.add_balance(target.get_uuid(), amount).await?;
                    let formatted = module_clone.format_money(amount);
                    sender.send_message(&format_text("&aGave {} to {}", formatted, target.get_name()));
                    target.send_message(&format_text("&aYou received {} from server", formatted));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("take")
                .permission("pumpkin-unified-core.eco.take")
                .argument("player")
                .argument("amount")
                .handler(async_handler!(move |sender, server, args| {
                    let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /eco take <player> <amount>"))?;
                    let amount_str = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /eco take <player> <amount>"))?;
                    let amount: f64 = amount_str.parse()?;
                    
                    let target = server.get_player(target_name)
                        .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
                    
                    module_clone.take_balance(target.get_uuid(), amount).await?;
                    let formatted = module_clone.format_money(amount);
                    sender.send_message(&format_text("&aTook {} from {}", formatted, target.get_name()));
                    target.send_message(&format_text("&c{} was taken from your balance", formatted));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("set")
                .permission("pumpkin-unified-core.eco.set")
                .argument("player")
                .argument("amount")
                .handler(async_handler!(move |sender, server, args| {
                    let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /eco set <player> <amount>"))?;
                    let amount_str = args.get(1).ok_or_else(|| anyhow::anyhow!("Usage: /eco set <player> <amount>"))?;
                    let amount: f64 = amount_str.parse()?;
                    
                    let target = server.get_player(target_name)
                        .ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
                    
                    module_clone.set_balance(target.get_uuid(), amount).await?;
                    let formatted = module_clone.format_money(amount);
                    sender.send_message(&format_text("&aSet {}'s balance to {}", target.get_name(), formatted));
                    target.send_message(&format_text("&aYour balance was set to {}", formatted));
                    Ok(())
                }))
                .build()
        )
        .build();
    context.register_command(eco_cmd);
}