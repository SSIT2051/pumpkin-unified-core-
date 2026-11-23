use crate::utils::macros::*;
use config::{Config as ConfigRs, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub economy: EconomyConfig,
    pub warps: WarpsConfig,
    pub homes: HomesConfig,
    pub tpa: TpaConfig,
    pub kits: KitsConfig,
    pub moderation: ModerationConfig,
    pub gamemode: GamemodeConfig,
    pub permissions: PermissionsConfig,
    pub spawn: SpawnConfig,
    pub utility: UtilityConfig,
    pub chat: ChatConfig,
    pub events: EventsConfig,
    pub scheduler: SchedulerConfig,
    pub holograms: HologramsConfig,
    pub portals: PortalsConfig,
    pub stafftools: StaffToolsConfig,
    pub interactiveblocks: InteractiveBlocksConfig,
    pub playertracker: PlayerTrackerConfig,
    pub chattab: ChatTabConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: String,
    pub sqlite: SqliteConfig,
    pub redis: RedisConfig,
    pub mysql: MysqlConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub path: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub starting_balance: f64,
    pub currency_symbol: String,
    pub currency_format: String,
    pub max_money: f64,
    pub min_money: f64,
    pub log_transactions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarpsConfig {
    pub max_warps_per_player: usize,
    pub warp_cooldown: u64,
    pub per_warp_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomesConfig {
    pub max_homes_default: usize,
    pub max_homes_vip: usize,
    pub home_cooldown: u64,
    pub home_on_death: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpaConfig {
    pub request_timeout: u64,
    pub teleport_cooldown: u64,
    pub cross_world: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KitsConfig {
    pub default_cooldown: u64,
    pub kit_signs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationConfig {
    pub ban_message: String,
    pub mute_message: String,
    pub jail_message: String,
    pub jail_location: Option<LocationConfig>,
    pub max_tempban_duration: u64,
    pub max_tempmute_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationConfig {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamemodeConfig {
    pub default: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsConfig {
    pub enable_contexts: bool,
    pub default_group: String,
    pub inheritance: InheritanceConfig,
    pub tracks: Vec<TrackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceConfig {
    pub max_depth: usize,
    pub order: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackConfig {
    pub name: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    pub spawn_location: Option<LocationConfig>,
    pub respawn_at_home: bool,
    pub bed_priority: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityConfig {
    pub afk_timeout: u64,
    pub afk_kick_timeout: Option<u64>,
    pub motd: String,
    pub rules: Vec<String>,
    pub max_players: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub format: String,
    pub broadcast_format: String,
    pub private_message_format: String,
    pub chat_radius: Option<f64>,
    pub anti_spam: bool,
    pub spam_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsConfig {
    pub player_join_message: Option<String>,
    pub player_leave_message: Option<String>,
    pub death_messages: bool,
    pub keep_inventory: bool,
    pub anti_build: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HologramsConfig {
    pub visibility_range: f64,
    pub max_holograms_per_player: usize,
    pub update_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalsConfig {
    pub max_portals: usize,
    pub default_cooldown: u64,
    pub particle_effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffToolsConfig {
    pub enable_vanish: bool,
    pub enable_social_spy: bool,
    pub vanish_on_login: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveBlocksConfig {
    pub elevator_material: String,
    pub armor_stand_editable: bool,
    pub command_signs_editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTrackerConfig {
    pub track_playtime: bool,
    pub track_ips: bool,
    pub track_first_join: bool,
    pub audit_log_enabled: bool,
    pub audit_log_blocks: bool,
    pub audit_log_items: bool,
    pub audit_log_commands: bool,
    pub max_audit_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTabConfig {
    pub format: String,
    pub tab_header: String,
    pub tab_footer: String,
    pub chat_channels: bool,
    pub hex_colors: bool,
    pub hover_tooltips: bool,
}

impl Config {
    pub fn load() -> Self {
        let config_dir =
            std::env::var("PUMPKIN_CONFIG_DIR").unwrap_or_else(|_| "config".to_string());
        let config_path = Path::new(&config_dir).join("pumpkin-unified-core");

        let mut builder = ConfigRs::builder()
            .add_source(
                File::from(config_path.join("config.default.toml"))
                    .format(FileFormat::Toml)
                    .required(false),
            )
            .add_source(
                File::from(config_path.join("config.default.yml"))
                    .format(FileFormat::Yaml)
                    .required(false),
            )
            .add_source(
                File::from(config_path.join("config.toml"))
                    .format(FileFormat::Toml)
                    .required(false),
            )
            .add_source(
                File::from(config_path.join("config.yml"))
                    .format(FileFormat::Yaml)
                    .required(false),
            );

        builder = builder.add_source(config::Environment::with_prefix("PUC"));

        let config_rs = builder.build().expect("Failed to build configuration");

        config_rs.try_deserialize().unwrap_or_else(|e| {
            tracing::warn!("Failed to deserialize config, using defaults: {}", e);
            Self::default()
        })
    }

    pub fn reload(&self) -> Self {
        Self::load()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                backend: "sqlite".to_string(),
                sqlite: SqliteConfig {
                    path: "plugins/pumpkin-unified-core/data.db".to_string(),
                    max_connections: 10,
                },
                redis: RedisConfig {
                    url: "redis://127.0.0.1:6379".to_string(),
                    max_connections: 10,
                },
                mysql: MysqlConfig {
                    url: "mysql://user:pass@localhost/pumpkin".to_string(),
                    max_connections: 10,
                },
            },
            economy: EconomyConfig {
                starting_balance: 100.0,
                currency_symbol: "$".to_string(),
                currency_format: "{symbol}{amount}".to_string(),
                max_money: 1_000_000_000.0,
                min_money: 0.0,
                log_transactions: true,
            },
            warps: WarpsConfig {
                max_warps_per_player: 10,
                warp_cooldown: 5,
                per_warp_permissions: true,
            },
            homes: HomesConfig {
                max_homes_default: 3,
                max_homes_vip: 10,
                home_cooldown: 5,
                home_on_death: false,
            },
            tpa: TpaConfig {
                request_timeout: 30,
                teleport_cooldown: 5,
                cross_world: true,
            },
            kits: KitsConfig {
                default_cooldown: 86400,
                kit_signs: true,
            },
            moderation: ModerationConfig {
                ban_message: "You have been banned from this server.".to_string(),
                mute_message: "You have been muted.".to_string(),
                jail_message: "You have been jailed.".to_string(),
                jail_location: None,
                max_tempban_duration: 2592000,
                max_tempmute_duration: 2592000,
            },
            gamemode: GamemodeConfig {
                default: "survival".to_string(),
            },
            permissions: PermissionsConfig {
                enable_contexts: true,
                default_group: "default".to_string(),
                inheritance: InheritanceConfig {
                    max_depth: 10,
                    order: "depth-first".to_string(),
                },
                tracks: vec![],
            },
            spawn: SpawnConfig {
                spawn_location: None,
                respawn_at_home: false,
                bed_priority: true,
            },
            utility: UtilityConfig {
                afk_timeout: 300,
                afk_kick_timeout: None,
                motd: "Welcome to the server!".to_string(),
                rules: vec!["Be respectful".to_string(), "No cheating".to_string()],
                max_players: 20,
            },
            chat: ChatConfig {
                format: "&8[&7{group}&8] &r{prefix}&r{player}&r: &f{message}".to_string(),
                broadcast_format: "&4[Broadcast] &r{message}".to_string(),
                private_message_format: "&8[&bPM&8] &r{from}&r -> &r{to}&r: &f{message}"
                    .to_string(),
                chat_radius: None,
                anti_spam: true,
                spam_threshold: 5,
            },
            events: EventsConfig {
                player_join_message: Some("&a{player} joined the game".to_string()),
                player_leave_message: Some("&c{player} left the game".to_string()),
                death_messages: true,
                keep_inventory: false,
                anti_build: true,
            },
scheduler: SchedulerConfig {
    enabled: true,
},
holograms: HologramsConfig {
    visibility_range: 48.0,
    max_holograms_per_player: 10,
    update_interval: 20,
},
portals: PortalsConfig {
    max_portals: 100,
    default_cooldown: 5,
    particle_effect: "portal".to_string(),
},
stafftools: StaffToolsConfig {
    enable_vanish: true,
    enable_social_spy: true,
    vanish_on_login: false,
},
interactiveblocks: InteractiveBlocksConfig {
    elevator_material: "white_wool".to_string(),
    armor_stand_editable: true,
    command_signs_editable: true,
},
playertracker: PlayerTrackerConfig {
    track_playtime: true,
    track_ips: true,
    track_first_join: true,
    audit_log_enabled: true,
    audit_log_blocks: true,
    audit_log_items: true,
    audit_log_commands: true,
    max_audit_entries: 1000,
},
chattab: ChatTabConfig {
    format: "&8[&7{group}&8] &r{prefix}&r{player}&r: &f{message}".to_string(),
    tab_header: "&6&lWelcome to &e{server}&6!&r\n&7Online: &e{online}/{max}".to_string(),
    tab_footer: "&7Website: &ehttps://example.com&r\n&7Discord: &ediscord.gg/example".to_string(),
    chat_channels: true,
    hex_colors: true,
    hover_tooltips: true,
},
        }
    }
}
