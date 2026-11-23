#[macro_export]
macro_rules! register_command {
    ($context:expr, $name:expr, $handler:expr, $permission:expr) => {{
        use pumpkin_plugin_api::command::{Command, CommandNode, CommandHandler};
        use pumpkin_plugin_api::permission::PermissionDefault;

        let command = Command::new($name)
            .permission($permission)
            .handler($handler)
            .build();

        $context.register_command(command);
    }};

    ($context:expr, $name:expr, $handler:expr, $permission:expr, $($arg:expr),+) => {{
        use pumpkin_plugin_api::command::{Command, CommandNode, CommandHandler};

        let mut command = Command::new($name)
            .permission($permission)
            .handler($handler);

        $(
            command = command.argument($arg);
        )+

        $context.register_command(command.build());
    }};
}

#[macro_export]
macro_rules! register_event_handler {
    ($context:expr, $handler:expr) => {{
        $context.register_event_handler($handler);
    }};
}

#[macro_export]
macro_rules! require_permission {
    ($sender:expr, $permission:expr) => {{
        if !$sender.has_permission($permission) {
            $sender.send_message(&format!("&cYou don't have permission: {}", $permission));
            return Ok(());
        }
    }};
}

#[macro_export]
macro_rules! parse_args {
    ($args:expr, $($name:ident: $type:ty),+) => {{
        let mut iter = $args.iter();
        ($(iter.next().and_then(|s| s.parse::<$type>().ok())),+)
    }};
}

#[macro_export]
macro_rules! format_message {
    ($($arg:tt)*) => {{
        use crate::utils::text::format_text;
        format_text(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! async_handler {
    ($handler:expr) => {
        move |sender, server, args| {
            let handler = $handler;
            Box::pin(async move { handler(sender, server, args).await })
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<(), anyhow::Error>> + Send>,
                >
        }
    };
}

#[macro_export]
macro_rules! async_handler_owned {
    ($handler:expr) => {
        Box::pin(async move { $handler.await })
            as std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<(), anyhow::Error>> + Send>,
            >
    };
}

pub use async_handler;
pub use format_message;
pub use parse_args;
pub use register_command;
pub use register_event_handler;
pub use require_permission;
