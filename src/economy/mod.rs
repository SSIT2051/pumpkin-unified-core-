use crate::storage::{SerializedData, Storage, StorageBackend};
use crate::utils::macros::*;
use crate::utils::text::format_text;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyBalance {
    pub uuid: Uuid,
    pub balance: f64,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyTransaction {
    pub id: Uuid,
    pub from_uuid: Option<Uuid>,
    pub to_uuid: Uuid,
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub timestamp: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Give,
    Take,
    Set,
    Pay,
    Interest,
    Kit,
    Shop,
}

pub struct EconomyModule {
    storage: Arc<StorageBackend>,
    cache: Arc<DashMap<Uuid, EconomyBalance>>,
    config: Arc<crate::config::EconomyConfig>,
}

impl EconomyModule {
    pub fn new(storage: Arc<StorageBackend>) -> Self {
        let config = Arc::new(crate::config::Config::default().economy);
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn with_config(
        storage: Arc<StorageBackend>,
        config: Arc<crate::config::EconomyConfig>,
    ) -> Self {
        Self {
            storage,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub async fn get_balance(&self, uuid: Uuid) -> Result<f64> {
        if let Some(cached) = self.cache.get(&uuid) {
            return Ok(cached.balance);
        }

        let key = uuid.to_string();
        if let Some(data) = self.storage.get("economy_balance", &key).await? {
            let balance_data: SerializedData<EconomyBalance> = SerializedData::deserialize(&data)?;
            let balance = balance_data.data.balance;
            self.cache.insert(uuid, balance_data.data);
            return Ok(balance);
        }

        let new_balance = EconomyBalance {
            uuid,
            balance: self.config.starting_balance,
            last_updated: chrono::Utc::now().timestamp(),
        };
        self.set_balance(uuid, self.config.starting_balance).await?;
        Ok(self.config.starting_balance)
    }

    pub async fn set_balance(&self, uuid: Uuid, amount: f64) -> Result<()> {
        let clamped = amount.clamp(self.config.min_money, self.config.max_money);
        let balance = EconomyBalance {
            uuid,
            balance: clamped,
            last_updated: chrono::Utc::now().timestamp(),
        };

        let data = SerializedData::new(balance.clone()).serialize()?;
        self.storage
            .set("economy_balance", &uuid.to_string(), &data)
            .await?;
        self.cache.insert(uuid, balance);
        Ok(())
    }

    pub async fn add_balance(&self, uuid: Uuid, amount: f64) -> Result<f64> {
        let current = self.get_balance(uuid).await?;
        let new_balance = (current + amount).clamp(self.config.min_money, self.config.max_money);
        self.set_balance(uuid, new_balance).await?;
        Ok(new_balance)
    }

    pub async fn take_balance(&self, uuid: Uuid, amount: f64) -> Result<f64> {
        self.add_balance(uuid, -amount).await
    }

    pub async fn pay(&self, from: Uuid, to: Uuid, amount: f64) -> Result<()> {
        if from == to {
            return Err(anyhow::anyhow!("Cannot pay yourself"));
        }

        let from_balance = self.get_balance(from).await?;
        if from_balance < amount {
            return Err(anyhow::anyhow!("Insufficient funds"));
        }

        self.take_balance(from, amount).await?;
        self.add_balance(to, amount).await?;

        if self.config.log_transactions {
            self.log_transaction(
                None,
                to,
                amount,
                TransactionType::Pay,
                Some(format!("Payment from {}", from)),
            )
            .await?;
            self.log_transaction(
                Some(from),
                to,
                amount,
                TransactionType::Pay,
                Some(format!("Payment to {}", to)),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn log_transaction(
        &self,
        from: Option<Uuid>,
        to: Uuid,
        amount: f64,
        tx_type: TransactionType,
        note: Option<String>,
    ) -> Result<()> {
        let transaction = EconomyTransaction {
            id: Uuid::new_v4(),
            from_uuid: from,
            to_uuid: to,
            amount,
            transaction_type: tx_type,
            timestamp: chrono::Utc::now().timestamp(),
            note,
        };

        let data = SerializedData::new(transaction).serialize()?;
        self.storage
            .set("economy_transactions", &transaction.id.to_string(), &data)
            .await?;
        Ok(())
    }

    pub async fn get_top_balances(&self, limit: usize) -> Result<Vec<(Uuid, f64)>> {
        let keys = self.storage.keys("economy_balance", "%").await?;
        let mut balances = Vec::new();

        for key in keys {
            if let Ok(uuid) = Uuid::parse_str(&key) {
                if let Some(data) = self.storage.get("economy_balance", &key).await? {
                    let balance_data: SerializedData<EconomyBalance> =
                        SerializedData::deserialize(&data)?;
                    balances.push((uuid, balance_data.data.balance));
                }
            }
        }

        balances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        balances.truncate(limit);
        Ok(balances)
    }

    pub fn format_money(&self, amount: f64) -> String {
        self.config
            .currency_format
            .replace("{symbol}", &self.config.currency_symbol)
            .replace("{amount}", &format!("{:.2}", amount))
    }

    pub fn register_commands(&self, context: &mut pumpkin_plugin_api::context::Context) {
        crate::economy::commands::register(self.clone(), context);
    }
}
