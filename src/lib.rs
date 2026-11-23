use pumpkin_plugin_api::{
    command::{Command, CommandHandler, CommandNode},
    context::Context,
    event::EventHandler,
    permission::{Permission, PermissionDefault},
    plugin::{Plugin, PluginMetadata},
    server::Server,
};
use std::sync::Arc;

mod chat;
mod chattab;
mod config;
mod economy;
mod events;
mod gamemode;
mod homes;
mod holograms;
mod interactiveblocks;
mod kits;
mod moderation;
mod permissions;
mod playertracker;
mod portals;
mod scheduler;
mod spawn;
mod stafftools;
mod storage;
mod tpa;
mod utility;
mod utils;
mod warps;

use crate::{
    chat::ChatModule,
    chattab::ChatTabModule,
    config::Config,
    economy::EconomyModule,
    events::EventModule,
    gamemode::GamemodeModule,
    homes::HomesModule,
    holograms::HologramsModule,
    interactiveblocks::InteractiveBlocksModule,
    kits::KitsModule,
    moderation::ModerationModule,
    permissions::PermissionsModule,
    playertracker::PlayerTrackerModule,
    portals::PortalsModule,
    scheduler::SchedulerModule,
    spawn::SpawnModule,
    stafftools::StaffToolsModule,
    storage::StorageManager,
    tpa::TpaModule,
    utility::UtilityModule,
    warps::WarpsModule,
};

pub struct UnifiedPlugin {
    config: Arc<Config>,
    storage: Arc<StorageManager>,
    economy: Arc<EconomyModule>,
    warps: Arc<WarpsModule>,
    homes: Arc<HomesModule>,
    tpa: Arc<TpaModule>,
    kits: Arc<KitsModule>,
    moderation: Arc<ModerationModule>,
    gamemode: Arc<GamemodeModule>,
    permissions: Arc<PermissionsModule>,
    spawn: Arc<SpawnModule>,
    utility: Arc<UtilityModule>,
    chat: Arc<ChatModule>,
    chattab: Arc<ChatTabModule>,
    holograms: Arc<HologramsModule>,
    portals: Arc<PortalsModule>,
    stafftools: Arc<StaffToolsModule>,
    interactiveblocks: Arc<InteractiveBlocksModule>,
    playertracker: Arc<PlayerTrackerModule>,
    scheduler: Arc<SchedulerModule>,
    events: Arc<EventModule>,
}

impl Plugin for UnifiedPlugin {
    fn new() -> Self {
        let config = Arc::new(Config::load());
        let storage = Arc::new(StorageManager::new(&config));

        let economy = Arc::new(EconomyModule::with_config(
            storage.clone(),
            config.economy.clone(),
        ));
        let warps = Arc::new(WarpsModule::with_config(
            storage.clone(),
            config.warps.clone(),
        ));
        let homes = Arc::new(HomesModule::with_config(
            storage.clone(),
            config.homes.clone(),
        ));
        let tpa = Arc::new(TpaModule::with_config(config.tpa.clone()));
        let kits = Arc::new(KitsModule::with_config(
            storage.clone(),
            config.kits.clone(),
        ));
        let moderation = Arc::new(ModerationModule::with_config(
            storage.clone(),
            config.moderation.clone(),
        ));
        let gamemode = Arc::new(GamemodeModule::new());
        let permissions = Arc::new(PermissionsModule::with_config(
            storage.clone(),
            config.permissions.clone(),
        ));
        let spawn = Arc::new(SpawnModule::with_config(
            storage.clone(),
            config.spawn.clone(),
        ));
        let utility = Arc::new(UtilityModule::with_config(config.utility.clone()));
        let chat = Arc::new(ChatModule::new().with_permissions(permissions.clone()));
        let chattab = Arc::new(ChatTabModule::new(config.chattab.clone()));
        let holograms = Arc::new(HologramsModule::with_config(
            storage.clone(),
            config.holograms.clone(),
        ));
        let portals = Arc::new(PortalsModule::new(storage.clone()));
        let stafftools = Arc::new(StaffToolsModule::new(storage.clone(), moderation.clone()));
        let interactiveblocks = Arc::new(InteractiveBlocksModule::new(storage.clone()));
        let playertracker = Arc::new(PlayerTrackerModule::new(storage.clone()));
        let scheduler = Arc::new(SchedulerModule::with_config(
            storage.clone(),
            config.scheduler.clone(),
        ));
        let events = Arc::new(EventModule::new(
            economy.clone(),
            warps.clone(),
            homes.clone(),
            tpa.clone(),
            kits.clone(),
            moderation.clone(),
            gamemode.clone(),
            permissions.clone(),
            spawn.clone(),
            utility.clone(),
            chat.clone(),
        ));

        Self {
            config,
            storage,
            economy,
            warps,
            homes,
            tpa,
            kits,
            moderation,
            gamemode,
            permissions,
            spawn,
            utility,
            chat,
            chattab,
            holograms,
            portals,
            stafftools,
            interactiveblocks,
            playertracker,
            scheduler,
            events,
        }
    }

    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "pumpkin-unified-core",
            version: "0.1.0",
            authors: vec!["SSIT2051"],
            description: "Unified EssentialsX + LuckPerms native Rust plugin for PumpkinMC",
            api_version: "0.1.0-dev",
        }
    }

    fn on_load(&mut self, context: &mut Context) {
        tracing::info!("Loading pumpkin-unified-core...");

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.storage.init().expect("Failed to initialize storage");
            })
        });

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.stafftools
                    .load_data()
                    .expect("Failed to load stafftools warnings");
            })
        });

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.scheduler.initialize().await.expect("Failed to initialize scheduler");
            })
        });

        self.economy.register_commands(context);
        self.warps.register_commands(context);
        self.homes.register_commands(context);
        self.tpa.register_commands(context);
        self.kits.register_commands(context);
        self.moderation.register_commands(context);
        self.gamemode.register_commands(context);
        self.permissions.register_commands(context);
        self.spawn.register_commands(context);
        self.utility.register_commands(context);
        self.chat.register_commands(context);
        self.chattab.register_commands(context);
        self.holograms.register_commands(context);
        self.portals.register_commands(context);
        self.stafftools.register_commands(context);
        self.interactiveblocks.register_commands(context);
        self.playertracker.register_commands(context);
        self.scheduler.register_commands(context);

        self.register_permissions(context);

        self.events.register_handlers(context);
        self.stafftools
            .register_events(context)
            .expect("Failed to register stafftools event handlers");

        tracing::info!("pumpkin-unified-core loaded successfully!");
    }

    fn on_unload(&mut self, _context: &mut Context) {
        tracing::info!("Unloading pumpkin-unified-core...");
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.scheduler.shutdown().await.ok();
            })
        });
        self.storage.shutdown();
    }
}

impl UnifiedPlugin {
    fn register_permissions(&self, context: &mut Context) {
        let perms = vec![
            ("pumpkin-unified-core.balance", PermissionDefault::Allow),
            ("pumpkin-unified-core.balance.others", PermissionDefault::Op),
            ("pumpkin-unified-core.pay", PermissionDefault::Allow),
            ("pumpkin-unified-core.eco", PermissionDefault::Op),
            ("pumpkin-unified-core.eco.give", PermissionDefault::Op),
            ("pumpkin-unified-core.eco.take", PermissionDefault::Op),
            ("pumpkin-unified-core.eco.set", PermissionDefault::Op),
            ("pumpkin-unified-core.warp", PermissionDefault::Allow),
            ("pumpkin-unified-core.warp.others", PermissionDefault::Op),
            ("pumpkin-unified-core.setwarp", PermissionDefault::Op),
            ("pumpkin-unified-core.delwarp", PermissionDefault::Op),
            ("pumpkin-unified-core.warps", PermissionDefault::Allow),
            ("pumpkin-unified-core.warpinfo", PermissionDefault::Allow),
            ("pumpkin-unified-core.home", PermissionDefault::Allow),
            ("pumpkin-unified-core.home.others", PermissionDefault::Op),
            ("pumpkin-unified-core.sethome", PermissionDefault::Allow),
            ("pumpkin-unified-core.delhome", PermissionDefault::Allow),
            ("pumpkin-unified-core.homes", PermissionDefault::Allow),
            ("pumpkin-unified-core.homes.limit.*", PermissionDefault::Op),
            ("pumpkin-unified-core.tpa", PermissionDefault::Allow),
            ("pumpkin-unified-core.tpahere", PermissionDefault::Allow),
            ("pumpkin-unified-core.tpaccept", PermissionDefault::Allow),
            ("pumpkin-unified-core.tpdeny", PermissionDefault::Allow),
            ("pumpkin-unified-core.tptoggle", PermissionDefault::Allow),
            ("pumpkin-unified-core.kit", PermissionDefault::Allow),
            ("pumpkin-unified-core.kit.*", PermissionDefault::Op),
            ("pumpkin-unified-core.kits", PermissionDefault::Allow),
            ("pumpkin-unified-core.createkit", PermissionDefault::Op),
            ("pumpkin-unified-core.showkit", PermissionDefault::Op),
            ("pumpkin-unified-core.kick", PermissionDefault::Op),
            ("pumpkin-unified-core.ban", PermissionDefault::Op),
            ("pumpkin-unified-core.tempban", PermissionDefault::Op),
            ("pumpkin-unified-core.unban", PermissionDefault::Op),
            ("pumpkin-unified-core.mute", PermissionDefault::Op),
            ("pumpkin-unified-core.unmute", PermissionDefault::Op),
            ("pumpkin-unified-core.jail", PermissionDefault::Op),
            ("pumpkin-unified-core.unjail", PermissionDefault::Op),
            ("pumpkin-unified-core.setjail", PermissionDefault::Op),
            ("pumpkin-unified-core.deljail", PermissionDefault::Op),
            ("pumpkin-unified-core.jails", PermissionDefault::Op),
            ("pumpkin-unified-core.fly", PermissionDefault::Op),
            ("pumpkin-unified-core.fly.others", PermissionDefault::Op),
            ("pumpkin-unified-core.god", PermissionDefault::Op),
            ("pumpkin-unified-core.god.others", PermissionDefault::Op),
            ("pumpkin-unified-core.heal", PermissionDefault::Op),
            ("pumpkin-unified-core.heal.others", PermissionDefault::Op),
            ("pumpkin-unified-core.feed", PermissionDefault::Op),
            ("pumpkin-unified-core.feed.others", PermissionDefault::Op),
            ("pumpkin-unified-core.vanish", PermissionDefault::Op),
            ("pumpkin-unified-core.gamemode", PermissionDefault::Op),
            (
                "pumpkin-unified-core.gamemode.survival",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.gamemode.creative",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.gamemode.adventure",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.gamemode.spectator",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.spawn", PermissionDefault::Allow),
            ("pumpkin-unified-core.setspawn", PermissionDefault::Op),
            ("pumpkin-unified-core.afk", PermissionDefault::Allow),
            ("pumpkin-unified-core.afk.others", PermissionDefault::Op),
            ("pumpkin-unified-core.rules", PermissionDefault::Allow),
            ("pumpkin-unified-core.motd", PermissionDefault::Allow),
            ("pumpkin-unified-core.list", PermissionDefault::Allow),
            ("pumpkin-unified-core.whois", PermissionDefault::Op),
            ("pumpkin-unified-core.speed", PermissionDefault::Op),
            ("pumpkin-unified-core.speed.others", PermissionDefault::Op),
            ("pumpkin-unified-core.compass", PermissionDefault::Allow),
            ("pumpkin-unified-core.top", PermissionDefault::Allow),
            ("pumpkin-unified-core.back", PermissionDefault::Allow),
            (
                "pumpkin-unified-core.back.ondeath",
                PermissionDefault::Allow,
            ),
            ("pumpkin-unified-core.seen", PermissionDefault::Allow),
            ("pumpkin-unified-core.repair", PermissionDefault::Op),
            ("pumpkin-unified-core.repair.others", PermissionDefault::Op),
            ("pumpkin-unified-core.disposal", PermissionDefault::Op),
            ("pumpkin-unified-core.workbench", PermissionDefault::Allow),
            ("pumpkin-unified-core.anvil", PermissionDefault::Allow),
            ("pumpkin-unified-core.enderchest", PermissionDefault::Allow),
            (
                "pumpkin-unified-core.enderchest.others",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.weather", PermissionDefault::Op),
            ("pumpkin-unified-core.time", PermissionDefault::Op),
            ("pumpkin-unified-core.lightning", PermissionDefault::Op),
            ("pumpkin-unified-core.butcher", PermissionDefault::Op),
            ("pumpkin-unified-core.extinguish", PermissionDefault::Op),
            ("pumpkin-unified-core.msg", PermissionDefault::Allow),
            ("pumpkin-unified-core.reply", PermissionDefault::Allow),
            ("pumpkin-unified-core.mail", PermissionDefault::Allow),
            ("pumpkin-unified-core.mail.send", PermissionDefault::Allow),
            ("pumpkin-unified-core.broadcast", PermissionDefault::Op),
            ("pumpkin-unified-core.broadcastworld", PermissionDefault::Op),
            ("pumpkin-unified-core.staff", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.*", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.vanish", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.invsee", PermissionDefault::Op),
            (
                "pumpkin-unified-core.staff.staffchat",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.staff.warn", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.warnings", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.delwarn", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.checkban", PermissionDefault::Op),
            (
                "pumpkin-unified-core.staff.checkmute",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.staff.freeze", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.unfreeze", PermissionDefault::Op),
            (
                "pumpkin-unified-core.staff.socialspy",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.staff.clearchat",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.staff.broadcaststaff",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.user.info", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.user.editor", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.user.promote",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.user.demote", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.user.showtracks",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.user.clear", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.user.clone", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.user.permission",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.user.parent", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.user.meta", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.group.info", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.group.editor",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.group.listmembers",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.group.setweight",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.group.setdisplayname",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.group.showtracks",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.group.clear", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.group.rename",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.group.clone", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.group.permission",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.group.parent",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.group.meta", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.track.createtrack",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.track.deletetrack",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.track.listtracks",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.lp.log.recent", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.log.search", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.log.notify", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.sync", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.info", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.editor", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.applyedits", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.verbose", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.tree", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.search", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.import", PermissionDefault::Op),
            ("pumpkin-unified-core.lp.export", PermissionDefault::Op),
            (
                "pumpkin-unified-core.lp.reloadconfig",
                PermissionDefault::Op,
            ),
            (
                "pumpkin-unified-core.lp.translations",
                PermissionDefault::Op,
            ),
            ("pumpkin-unified-core.scheduler.create", PermissionDefault::Op),
            ("pumpkin-unified-core.scheduler.remove", PermissionDefault::Op),
            ("pumpkin-unified-core.scheduler.list", PermissionDefault::Op),
            ("pumpkin-unified-core.scheduler.enable", PermissionDefault::Op),
            ("pumpkin-unified-core.scheduler.disable", PermissionDefault::Op),
            ("pumpkin-unified-core.scheduler.run", PermissionDefault::Op),
            ("pumpkin-unified-core.scheduler.edit", PermissionDefault::Op),
            ("pumpkin-unified-core.scheduler.reload", PermissionDefault::Op),
            // Holograms (HoloManager)
            ("pumpkin-unified-core.hologram.create", PermissionDefault::Op),
            ("pumpkin-unified-core.hologram.remove", PermissionDefault::Op),
            ("pumpkin-unified-core.hologram.list", PermissionDefault::Allow),
            ("pumpkin-unified-core.hologram.teleport", PermissionDefault::Op),
            ("pumpkin-unified-core.hologram.edit", PermissionDefault::Op),
            ("pumpkin-unified-core.hologram.move", PermissionDefault::Op),
            ("pumpkin-unified-core.hologram.copy", PermissionDefault::Op),
            ("pumpkin-unified-core.hologram.readtext", PermissionDefault::Op),
            // Portals (PortalBuilder)
            ("pumpkin-unified-core.portal.create", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.remove", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.list", PermissionDefault::Allow),
            ("pumpkin-unified-core.portal.setdestination", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.setcommand", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.setparticle", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.setmaterial", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.setcooldown", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.setpermission", PermissionDefault::Op),
            ("pumpkin-unified-core.portal.teleport", PermissionDefault::Op),
            // StaffTools (StaffTools/ModSuite)
            ("pumpkin-unified-core.staff.vanish", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.invsee", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.staffchat", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.warn", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.warnings", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.delwarn", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.checkban", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.checkmute", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.freeze", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.unfreeze", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.socialspy", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.clearchat", PermissionDefault::Op),
            ("pumpkin-unified-core.staff.broadcaststaff", PermissionDefault::Op),
            // InteractiveBlocks (Elevator + ArmorStandTools)
            ("pumpkin-unified-core.interactive.elevator.create", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.elevator.remove", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.elevator.list", PermissionDefault::Allow),
            ("pumpkin-unified-core.interactive.elevator.setfloor", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.elevator.setmaterial", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.armorstand.pose", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.armorstand.remove", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.armorstand.list", PermissionDefault::Allow),
            ("pumpkin-unified-core.interactive.commandsign.create", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.commandsign.remove", PermissionDefault::Op),
            ("pumpkin-unified-core.interactive.commandsign.list", PermissionDefault::Allow),
            // PlayerTracker (PlayerTracker/AuditLog)
            ("pumpkin-unified-core.tracker.playtime", PermissionDefault::Allow),
            ("pumpkin-unified-core.tracker.playtime.top", PermissionDefault::Op),
            ("pumpkin-unified-core.tracker.seen", PermissionDefault::Allow),
            ("pumpkin-unified-core.tracker.firstjoin", PermissionDefault::Op),
            ("pumpkin-unified-core.tracker.iphistory", PermissionDefault::Op),
            ("pumpkin-unified-core.tracker.session", PermissionDefault::Op),
            ("pumpkin-unified-core.tracker.auditlog", PermissionDefault::Op),
            ("pumpkin-unified-core.tracker.auditlog.block", PermissionDefault::Op),
            ("pumpkin-unified-core.tracker.auditlog.item", PermissionDefault::Op),
            ("pumpkin-unified-core.tracker.auditlog.command", PermissionDefault::Op),
            // ChatTab (Chat & Tab Formatter)
            ("pumpkin-unified-core.chattab.format", PermissionDefault::Op),
            ("pumpkin-unified-core.chattab.tab", PermissionDefault::Op),
            ("pumpkin-unified-core.chattab.channel", PermissionDefault::Allow),
            ("pumpkin-unified-core.chattab.hex", PermissionDefault::Allow),
            ("pumpkin-unified-core.chattab.hover", PermissionDefault::Allow),
        ];

        for (name, default) in perms {
            context.register_permission(Permission::new(name, default));
        }
    }
}

pumpkin_plugin_api::register_plugin!(UnifiedPlugin);
