use crate::chat::ChatModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub fn register(module: Arc<ChatModule>, context: &mut Context) {
    let module_clone = module.clone();
    let msg_cmd = Command::new("msg")
        .alias("tell")
        .alias("w")
        .permission("pumpkin-unified-core.msg")
        .description("Send a private message")
        .argument("player")
        .argument("message")
        .handler(async_handler!(move |sender, server, args| {
            let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /msg <player> <message>"))?;
            let message = args.get(1..).map(|s| s.join(" ")).unwrap_or_default();
            
            let target = server.get_player(target_name).ok_or_else(|| anyhow::anyhow!("Player not found: {}", target_name))?;
            
            let format = module_clone.config.private_message_format
                .replace("{from}", sender.get_name())
                .replace("{to}", target.get_name())
                .replace("{message}", &message);
            
            sender.send_message(&format_text(&format));
            target.send_message(&format_text(&format));
            Ok(())
        }))
        .build();
    context.register_command(msg_cmd);

    let module_clone = module.clone();
    let r_cmd = Command::new("r")
        .alias("reply")
        .permission("pumpkin-unified-core.reply")
        .description("Reply to last private message")
        .argument("message")
        .handler(async_handler!(move |sender, _server, args| {
            let message = args.get(0..).map(|s| s.join(" ")).unwrap_or_default();
            sender.send_message(&format_text("&cReply command not fully implemented (no last message tracking)"));
            Ok(())
        }))
        .build();
    context.register_command(r_cmd);

    let module_clone = module.clone();
    let mail_cmd = Command::new("mail")
        .permission("pumpkin-unified-core.mail")
        .description("Manage offline messages")
        .subcommand(
            Command::new("send")
                .permission("pumpkin-unified-core.mail.send")
                .argument("player")
                .argument("message")
                .handler(async_handler!(move |sender, _server, args| {
                    let target_name = args.get(0).ok_or_else(|| anyhow::anyhow!("Usage: /mail send <player> <message>"))?;
                    let message = args.get(1..).map(|s| s.join(" ")).unwrap_or_default();
                    sender.send_message(&format_text("&cMail send not fully implemented"));
                    Ok(())
                }))
                .build()
        )
        .subcommand(
            Command::new("read")
                .permission("pumpkin-unified-core.mail")
                .handler(async_handler!(move |sender, _server, _args| {
                    sender.send_message(&format_text("&cMail read not fully implemented"));
                    Ok(())
                }))
                .build()
        )
        .build();
    context.register_command(mail_cmd);

    let module_clone = module.clone();
    let broadcast_cmd = Command::new("broadcast")
        .alias("bc")
        .permission("pumpkin-unified-core.broadcast")
        .description("Broadcast a message to all players")
        .argument("message")
        .handler(async_handler!(move |sender, server, args| {
            let message = args.get(0..).map(|s| s.join(" ")).unwrap_or_default();
            let format = module_clone.config.broadcast_format.replace("{message}", &message);
            
            for player in server.get_online_players() {
                player.send_message(&format_text(&format));
            }
            Ok(())
        }))
        .build();
    context.register_command(broadcast_cmd);

    let module_clone = module.clone();
    let broadcastworld_cmd = Command::new("broadcastworld")
        .alias("bcw")
        .permission("pumpkin-unified-core.broadcastworld")
        .description("Broadcast a message to players in the same world")
        .argument("message")
        .handler(async_handler!(move |sender, server, args| {
            let message = args.get(0..).map(|s| s.join(" ")).unwrap_or_default();
            let player = sender.as_player().ok_or_else(|| anyhow::anyhow!("Only players can use this command"))?;
            let world = player.get_world();
            let format = module_clone.config.broadcast_format.replace("{message}", &message);
            
            for p in world.get_players() {
                p.send_message(&format_text(&format));
            }
            Ok(())
        }))
        .build();
    context.register_command(broadcastworld_cmd);
}