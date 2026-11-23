use crate::scheduler::SchedulerModule;
use crate::utils::macros::*;
use crate::utils::text::format_text;
use pumpkin_plugin_api::{
    command::{Command, CommandSender},
    context::Context,
    server::Server,
};
use std::sync::Arc;

pub fn register(module: Arc<SchedulerModule>, context: &mut Context) {
    let module_clone = module.clone();
    let create_cmd = Command::new("scheduler")
        .alias("schedule")
        .permission("pumpkin-unified-core.scheduler.create")
        .description("Create a new scheduled task")
        .argument("create")
        .argument("name")
        .argument("cron")
        .argument("command")
        .handler(async_handler!(move |sender, _server, args| {
            if args.len() < 4 {
                sender.send_message(&format_text("&cUsage: /scheduler create <name> <cron> <command>"));
                sender.send_message(&format_text("&7Example: /scheduler create backup \"0 3 * * *\" \"save-all\""));
                sender.send_message(&format_text("&7Example: /scheduler create restart \"0 4 * * 0\" \"restart\""));
                return Ok(());
            }

            let name = args[1].to_string();
            let cron = args[2].to_string();
            let command = args[3..].join(" ");

            if !cron::Schedule::from_str(&cron).is_ok() {
                sender.send_message(&format_text("&cInvalid cron expression: {}", cron));
                sender.send_message(&format_text("&7Format: minute hour day month day-of-week"));
                sender.send_message(&format_text("&7Example: \"0 3 * * *\" = 3:00 AM daily"));
                sender.send_message(&format_text("&7Example: \"0 */6 * * *\" = Every 6 hours"));
                return Ok(());
            }

            let task = crate::scheduler::ScheduledTask::new(name.clone(), cron.clone(), command.clone());
            match module_clone.create_task(task).await {
                Ok(_) => {
                    sender.send_message(&format_text("&aCreated scheduled task: &e{} &7({})", name, cron));
                }
                Err(e) => {
                    sender.send_message(&format_text("&cFailed to create task: {}", e));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(create_cmd);

    let module_clone = module.clone();
    let remove_cmd = Command::new("scheduler")
        .permission("pumpkin-unified-core.scheduler.remove")
        .description("Remove a scheduled task")
        .argument("remove")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            if args.len() < 2 {
                sender.send_message(&format_text("&cUsage: /scheduler remove <name>"));
                return Ok(());
            }

            let name = args[1].to_string();
            match module_clone.remove_task(&name).await {
                Ok(_) => {
                    sender.send_message(&format_text("&aRemoved scheduled task: &e{}", name));
                }
                Err(e) => {
                    sender.send_message(&format_text("&cFailed to remove task: {}", e));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(remove_cmd);

    let module_clone = module.clone();
    let list_cmd = Command::new("scheduler")
        .permission("pumpkin-unified-core.scheduler.list")
        .description("List all scheduled tasks")
        .argument("list")
        .handler(async_handler!(move |sender, _server, _args| {
            let tasks = module_clone.list_tasks().await;
            if tasks.is_empty() {
                sender.send_message(&format_text("&cNo scheduled tasks found"));
            } else {
                sender.send_message(&format_text("&6&l--- Scheduled Tasks ---"));
                for task in tasks {
                    let status = if task.enabled { "&aEnabled" } else { "&cDisabled" };
                    let last_run = task.last_run.map(|t| {
                        chrono::DateTime::from_timestamp(t, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    }).unwrap_or_else(|| "&7Never".to_string());
                    
                    let next_run = task.next_run.map(|t| {
                        chrono::DateTime::from_timestamp(t, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    }).unwrap_or_else(|| "&7Never".to_string());

                    let recurrence = if task.one_time { "&e[One-time]" } else { "&7[Recurring]" };
                    
                    sender.send_message(&format_text(
                        "&e{} &7| {} &7| Last: {} &7| Next: {} &7| Command: &f{}",
                        task.name, status, last_run, next_run, task.command
                    ));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(list_cmd);

    let module_clone = module.clone();
    let enable_cmd = Command::new("scheduler")
        .permission("pumpkin-unified-core.scheduler.enable")
        .description("Enable a scheduled task")
        .argument("enable")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            if args.len() < 2 {
                sender.send_message(&format_text("&cUsage: /scheduler enable <name>"));
                return Ok(());
            }

            let name = args[1].to_string();
            match module_clone.enable_task(&name).await {
                Ok(_) => {
                    sender.send_message(&format_text("&aEnabled scheduled task: &e{}", name));
                }
                Err(e) => {
                    sender.send_message(&format_text("&cFailed to enable task: {}", e));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(enable_cmd);

    let module_clone = module.clone();
    let disable_cmd = Command::new("scheduler")
        .permission("pumpkin-unified-core.scheduler.disable")
        .description("Disable a scheduled task")
        .argument("disable")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            if args.len() < 2 {
                sender.send_message(&format_text("&cUsage: /scheduler disable <name>"));
                return Ok(());
            }

            let name = args[1].to_string();
            match module_clone.disable_task(&name).await {
                Ok(_) => {
                    sender.send_message(&format_text("&aDisabled scheduled task: &e{}", name));
                }
                Err(e) => {
                    sender.send_message(&format_text("&cFailed to disable task: {}", e));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(disable_cmd);

    let module_clone = module.clone();
    let run_cmd = Command::new("scheduler")
        .permission("pumpkin-unified-core.scheduler.run")
        .description("Run a scheduled task immediately")
        .argument("run")
        .argument("name")
        .handler(async_handler!(move |sender, _server, args| {
            if args.len() < 2 {
                sender.send_message(&format_text("&cUsage: /scheduler run <name>"));
                return Ok(());
            }

            let name = args[1].to_string();
            match module_clone.run_task(&name).await {
                Ok(_) => {
                    sender.send_message(&format_text("&aExecuted scheduled task: &e{}", name));
                }
                Err(e) => {
                    sender.send_message(&format_text("&cFailed to run task: {}", e));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(run_cmd);

    let module_clone = module.clone();
    let edit_cmd = Command::new("scheduler")
        .permission("pumpkin-unified-core.scheduler.edit")
        .description("Edit a scheduled task")
        .argument("edit")
        .argument("name")
        .argument("cron")
        .argument("command")
        .handler(async_handler!(move |sender, _server, args| {
            if args.len() < 4 {
                sender.send_message(&format_text("&cUsage: /scheduler edit <name> <cron> <command>"));
                return Ok(());
            }

            let name = args[1].to_string();
            let cron = args[2].to_string();
            let command = args[3..].join(" ");

            if !cron::Schedule::from_str(&cron).is_ok() {
                sender.send_message(&format_text("&cInvalid cron expression: {}", cron));
                return Ok(());
            }

            match module_clone.edit_task(&name, cron, command).await {
                Ok(_) => {
                    sender.send_message(&format_text("&aEdited scheduled task: &e{}", name));
                }
                Err(e) => {
                    sender.send_message(&format_text("&cFailed to edit task: {}", e));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(edit_cmd);

    let module_clone = module.clone();
    let reload_cmd = Command::new("scheduler")
        .permission("pumpkin-unified-core.scheduler.reload")
        .description("Reload all scheduled tasks from storage")
        .argument("reload")
        .handler(async_handler!(move |sender, _server, _args| {
            match module_clone.reload().await {
                Ok(_) => {
                    sender.send_message(&format_text("&aReloaded scheduled tasks from storage"));
                }
                Err(e) => {
                    sender.send_message(&format_text("&cFailed to reload tasks: {}", e));
                }
            }
            Ok(())
        }))
        .build();
    context.register_command(reload_cmd);
}