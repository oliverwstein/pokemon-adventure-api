// This file contains shared helper code for all integration tests.
// It will not be included in the final production binary.

use anyhow::anyhow;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{
    database::Db,
    handlers::BattleHandler,
    types::{BattleId, StoredBattle},
    ApiError,
};
// --- MOCK DATABASE ---
#[derive(Clone)]
pub struct MockDb {
    battles: Arc<Mutex<HashMap<BattleId, StoredBattle>>>,
}

pub fn create_test_handler() -> Result<BattleHandler, ApiError> {
    let mock_db = MockDb::new();
    // Call the simple `new` constructor, not the async one.
    Ok(BattleHandler::new(Arc::new(mock_db)))
}

impl MockDb {
    pub fn new() -> Self {
        Self {
            battles: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Db for MockDb {
    async fn create_battle(&self, battle: &StoredBattle) -> Result<(), anyhow::Error> {
        let mut battles = self.battles.lock().unwrap();
        if battles.contains_key(&battle.battle_id) {
            return Err(anyhow!("Battle already exists"));
        }
        battles.insert(battle.battle_id, battle.clone());
        Ok(())
    }

    async fn get_battle(
        &self,
        battle_id: BattleId,
    ) -> Result<Option<StoredBattle>, anyhow::Error> {
        let battles = self.battles.lock().unwrap();
        Ok(battles.get(&battle_id).cloned())
    }

    async fn update_battle(&self, battle: &StoredBattle) -> Result<(), anyhow::Error> {
        let mut battles = self.battles.lock().unwrap();
        if !battles.contains_key(&battle.battle_id) {
            return Err(anyhow!("Battle not found"));
        }
        battles.insert(battle.battle_id, battle.clone());
        Ok(())
    }
}