use crate::storage::{StorageBackend, SerializedData};
use crate::utils::macros::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use cron::Schedule;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub name: String,
    pub cron: String,
    pub command: String,
    pub enabled: bool,
    pub last_run: Option<i64>,
    pub next_run: Option<i64>,
    pub one_time: bool,
    pub created_at: i64,
}

impl ScheduledTask {
    pub fn new(name: String, cron: String, command: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4(),
            name,
            cron,
            command,
            enabled: true,
            last_run: None,
            next_run: None,
            one_time: false,
            created_at: now,
        }
    }

    pub fn new_one_time(name: String, cron: String, command: String) -> Self {
        let mut task = Self::new(name, cron, command);
        task.one_time = true;
        task
    }

    fn cache_key(&self) -> String {
        format!("task:{}", self.name.to_lowercase())
    }

    fn calculate_next_run(&self) -> Option<i64> {
        if self.one_time {
            if let Ok(schedule) = Schedule::from_str(&self.cron) {
                if let Some(next) = schedule.upcoming(Utc).next() {
                    return Some(next.timestamp());
                }
            }
            return None;
        }

        if let Ok(schedule) = Schedule::from_str(&self.cron) {
            if let Some(next) = schedule.upcoming(Utc).next() {
                return Some(next.timestamp());
            }
        }
        None
    }
}

pub struct SchedulerModule {
    storage: Arc<StorageBackend>,
    cache: Arc<DashMap<String, ScheduledTask>>,
    scheduler: Arc<JobScheduler>,
    config: Arc<crate::config::SchedulerConfig>,
}

impl SchedulerModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().scheduler);
        let scheduler = Arc::new(JobScheduler::new().expect("Failed to create job scheduler"));
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            scheduler,
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::SchedulerConfig>,
    ) -> Self {
        let scheduler = Arc::new(JobScheduler::new().expect("Failed to create job scheduler"));
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            scheduler,
            config,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Scheduler module is disabled");
            return Ok(());
        }

        self.load_tasks().await?;
        self.scheduler.start().await?;
        tracing::info!("Scheduler module initialized with {} tasks", self.cache.len());
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.scheduler.shutdown().await?;
        Ok(())
    }

    async fn load_tasks(&self) -> Result<()> {
        let keys = self.storage.keys("scheduler_tasks", "task:*").await?;
        for key in keys {
            if let Some(data) = self.storage.get("scheduler_tasks", &key).await? {
                let task_data: SerializedData<ScheduledTask> =
                    SerializedData::deserialize(&data)?;
                let task = task_data.data;

                if task.enabled {
                    self.schedule_task(&task).await?;
                }

                let mut task_with_next = task.clone();
                task_with_next.next_run = task_with_next.calculate_next_run();
                self.cache.insert(task_with_next.cache_key(), task_with_next);
            }
        }
        Ok(())
    }

    async fn save_task(&self, task: &ScheduledTask) -> Result<()> {
        let data = SerializedData::new(task.clone()).serialize()?;
        self.storage
            .set("scheduler_tasks", &task.cache_key(), &data)
            .await?;
        Ok(())
    }

    async fn delete_task(&self, name: &str) -> Result<()> {
        let key = format!("task:{}", name.to_lowercase());
        self.storage.delete("scheduler_tasks", &key).await?;
        self.cache.remove(&key);
        Ok(())
    }

    async fn schedule_task(&self, task: &ScheduledTask) -> Result<()> {
        let job_id = format!("scheduler-{}", task.name);
        let cron_expr = &task.cron;

        let module_clone = self.clone();
        let task_name = task.name.clone();
        let command = task.command.clone();
        let one_time = task.one_time;

        let job = Job::new_async(cron_expr, move |_uuid, _l| {
            let module = module_clone.clone();
            let task_name = task_name.clone();
            let command = command.clone();
            Box::pin(async move {
                module.execute_task(&task_name, &command).await;
                if one_time {
                    module.remove_task(&task_name).await.ok();
                }
            })
        })?
        .with_id(job_id.parse()?);

        self.scheduler.add(job).await?;
        Ok(())
    }

    async fn unschedule_task(&self, name: &str) -> Result<()> {
        let job_id = format!("scheduler-{}", name);
        self.scheduler.remove(&job_id.parse()?).await?;
        Ok(())
    }

    pub async fn create_task(&self, task: ScheduledTask) -> Result<()> {
        if self.cache.contains_key(&task.cache_key()) {
            return Err(anyhow::anyhow!("Task with name '{}' already exists", task.name));
        }

        if task.enabled {
            self.schedule_task(&task).await?;
        }

        let mut task_with_next = task.clone();
        task_with_next.next_run = task_with_next.calculate_next_run();
        self.save_task(&task_with_next).await?;
        self.cache.insert(task.cache_key(), task_with_next);
        Ok(())
    }

    pub async fn remove_task(&self, name: &str) -> Result<()> {
        let key = format!("task:{}", name.to_lowercase());
        if let Some((_, task)) = self.cache.remove(&key) {
            if task.enabled {
                self.unschedule_task(name).await?;
            }
            self.delete_task(name).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}", name))
        }
    }

    pub async fn enable_task(&self, name: &str) -> Result<()> {
        let key = format!("task:{}", name.to_lowercase());
        if let Some(mut task) = self.cache.get_mut(&key) {
            if !task.enabled {
                task.enabled = true;
                self.schedule_task(&task).await?;
                task.next_run = task.calculate_next_run();
                self.save_task(&task).await?;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}", name))
        }
    }

    pub async fn disable_task(&self, name: &str) -> Result<()> {
        let key = format!("task:{}", name.to_lowercase());
        if let Some(mut task) = self.cache.get_mut(&key) {
            if task.enabled {
                task.enabled = false;
                self.unschedule_task(name).await?;
                task.next_run = None;
                self.save_task(&task).await?;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}", name))
        }
    }

    pub async fn run_task(&self, name: &str) -> Result<()> {
        let key = format!("task:{}", name.to_lowercase());
        if let Some(task) = self.cache.get(&key) {
            self.execute_task(&task.name, &task.command).await;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}", name))
        }
    }

    pub async fn edit_task(&self, name: &str, cron: String, command: String) -> Result<()> {
        let key = format!("task:{}", name.to_lowercase());
        if let Some(mut task) = self.cache.get_mut(&key) {
            let was_enabled = task.enabled;

            if was_enabled {
                self.unschedule_task(name).await?;
            }

            task.cron = cron;
            task.command = command;

            if was_enabled {
                self.schedule_task(&task).await?;
            }

            task.next_run = task.calculate_next_run();
            self.save_task(&task).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}", name))
        }
    }

    pub async fn reload(&self) -> Result<()> {
        self.shutdown().await?;
        self.scheduler.start().await?;
        self.cache.clear();
        self.load_tasks().await?;
        Ok(())
    }

    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        self.cache.iter().map(|entry| entry.clone()).collect()
    }

    pub async fn get_task(&self, name: &str) -> Option<ScheduledTask> {
        let key = format!("task:{}", name.to_lowercase());
        self.cache.get(&key).map(|entry| entry.clone())
    }

    async fn execute_task(&self, name: &str, command_template: &str) {
        let server = crate::get_server().await;
        if server.is_none() {
            tracing::warn!("Cannot execute scheduled task '{}': server not available", name);
            return;
        }

        let server = server.unwrap();
        let online_players = server.get_online_players().await.len();

        let mut placeholders = std::collections::HashMap::new();
        placeholders.insert("server".to_string(), server.get_name());
        placeholders.insert("time".to_string(), chrono::Local::now().format("%H:%M:%S").to_string());
        placeholders.insert("date".to_string(), chrono::Local::now().format("%Y-%m-%d").to_string());
        placeholders.insert("online_players".to_string(), online_players.to_string());

        let command = crate::utils::text::replace_placeholders(command_template, &placeholders);

        tracing::info!("Executing scheduled task '{}': {}", name, command);

        if let Err(e) = server.dispatch_command(&command).await {
            tracing::error!("Failed to execute scheduled task '{}': {}", name, e);
        }

        if let Some(mut task) = self.cache.get_mut(&format!("task:{}", name.to_lowercase())) {
            let now = Utc::now().timestamp();
            task.last_run = Some(now);
            task.next_run = task.calculate_next_run();
            self.save_task(&task).await.ok();
        }
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::scheduler::commands::register(self.clone(), context);
    }
}

impl Clone for SchedulerModule {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            cache: self.cache.clone(),
            scheduler: self.scheduler.clone(),
            config: self.config.clone(),
        }
    }
}

async fn get_server() -> Option<pumpkin_plugin_api::server::Server> {
    pumpkin_plugin_api::server::Server::get().ok()
}