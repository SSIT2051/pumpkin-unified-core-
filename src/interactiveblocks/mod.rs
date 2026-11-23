use crate::storage::StorageBackend;
use crate::utils::macros::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevatorFloor {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Elevator {
    pub id: Uuid,
    pub name: String,
    pub material: String,
    pub floors: Vec<ElevatorFloor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmorStandPose {
    pub name: String,
    pub head: (f32, f32, f32),
    pub body: (f32, f32, f32),
    pub left_arm: (f32, f32, f32),
    pub right_arm: (f32, f32, f32),
    pub left_leg: (f32, f32, f32),
    pub right_leg: (f32, f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSign {
    pub id: Uuid,
    pub name: String,
    pub command: String,
}

pub struct InteractiveBlocksModule {
    storage: Arc<StorageBackend>,
    elevators: Arc<DashMap<String, Elevator>>,
    armor_stands: Arc<DashMap<String, ArmorStandPose>>,
    command_signs: Arc<DashMap<String, CommandSign>>,
}

impl InteractiveBlocksModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        Self {
            storage,
            elevators: Arc::new(DashMap::new()),
            armor_stands: Arc::new(DashMap::new()),
            command_signs: Arc::new(DashMap::new()),
        }
    }

    pub async fn save_elevator(&self, elevator: Elevator) -> Result<()> {
        self.elevators.insert(elevator.name.clone(), elevator.clone());
        let data = crate::storage::SerializedData::new(elevator.clone());
        self.storage.set("interactiveblocks_elevators", &elevator.name.to_lowercase(), &data.serialize()?).await?;
        Ok(())
    }

    pub async fn remove_elevator(&self, name: &str) -> Result<()> {
        self.elevators.remove(name);
        self.storage.delete("interactiveblocks_elevators", &name.to_lowercase()).await?;
        Ok(())
    }

    pub async fn list_elevators(&self) -> Vec<Elevator> {
        self.elevators.iter().map(|e| e.value().clone()).collect()
    }

    pub async fn save_armor_stand(&self, pose: ArmorStandPose) -> Result<()> {
        self.armor_stands.insert(pose.name.clone(), pose.clone());
        let data = crate::storage::SerializedData::new(pose.clone());
        self.storage.set("interactiveblocks_armorstands", &pose.name.to_lowercase(), &data.serialize()?).await?;
        Ok(())
    }

    pub async fn remove_armor_stand(&self, name: &str) -> Result<()> {
        self.armor_stands.remove(name);
        self.storage.delete("interactiveblocks_armorstands", &name.to_lowercase()).await?;
        Ok(())
    }

    pub async fn list_armor_stands(&self) -> Vec<ArmorStandPose> {
        self.armor_stands.iter().map(|a| a.value().clone()).collect()
    }

    pub async fn save_command_sign(&self, sign: CommandSign) -> Result<()> {
        self.command_signs.insert(sign.name.clone(), sign.clone());
        let data = crate::storage::SerializedData::new(sign.clone());
        self.storage.set("interactiveblocks_commandsigns", &sign.name.to_lowercase(), &data.serialize()?).await?;
        Ok(())
    }

    pub async fn remove_command_sign(&self, name: &str) -> Result<()> {
        self.command_signs.remove(name);
        self.storage.delete("interactiveblocks_commandsigns", &name.to_lowercase()).await?;
        Ok(())
    }

    pub async fn list_command_signs(&self) -> Vec<CommandSign> {
        self.command_signs.iter().map(|c| c.value().clone()).collect()
    }
}
