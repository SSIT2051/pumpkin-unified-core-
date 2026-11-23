use std::sync::Weak;
use std::sync::atomic::Ordering;

use super::{Controls, Goal};

use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::mob::creeper::CreeperEntity;

pub struct CreeperIgniteGoal {
    goal_control: Controls,
    creeper: Weak<CreeperEntity>,
    target: Option<Weak<dyn EntityBase>>,
}

impl CreeperIgniteGoal {
    #[must_use]
    pub const fn new(creeper: Weak<CreeperEntity>) -> Self {
        Self {
            goal_control: Controls::MOVE,
            creeper,
            target: None,
        }
    }
}

impl Goal for CreeperIgniteGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(creeper) = self.creeper.upgrade() else {
            return false;
        };
        if creeper.fuse_speed.load(Ordering::Relaxed) > 0 {
            return true;
        }

        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        target.get_entity().is_alive()
            && mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load())
                < 9.0
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().steered().navigator().stop();
        self.target = mob
            .get_mob_entity()
            .get_target()
            .as_ref()
            .map(std::sync::Arc::downgrade);
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(creeper) = self.creeper.upgrade() else {
            return;
        };
        let Some(target) = self
            .target
            .as_ref()
            .and_then(Weak::upgrade)
            .filter(|t| t.get_entity().is_alive())
        else {
            creeper.set_fuse_speed(-1);
            return;
        };

        let dist_sq = mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&target.get_entity().pos.load());

        if dist_sq > 49.0 || !mob.has_line_of_sight(target.get_entity()) {
            creeper.set_fuse_speed(-1);
        } else if creeper.fuse_speed.load(Ordering::Relaxed) <= 0 {
            // Swell starting - emit PRIME_FUSE game event
            let entity = &creeper.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_game_event(
                crate::world::GameEvent::PrimeFuse,
                pos.into(),
                Some(entity),
            );
            creeper.set_fuse_speed(1);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
