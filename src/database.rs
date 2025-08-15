use aws_sdk_dynamodb::{Client, Error as DynamoError};
use aws_sdk_dynamodb::types::AttributeValue;
use serde_json;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{BattleId, PlayerId, StoredBattle};

pub struct Database {
    client: Client,
    table_name: String,
}

impl Database {
    pub async fn new(table_name: String) -> Result<Self, anyhow::Error> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        
        Ok(Database {
            client,
            table_name,
        })
    }

    /// Create a new battle in the database
    pub async fn create_battle(&self, battle: &StoredBattle) -> Result<(), anyhow::Error> {
        let item = self.battle_to_item(battle)?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(battle_id)") // Prevent overwrites
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create battle: {}", e))?;

        Ok(())
    }

    /// Retrieve a battle from the database
    pub async fn get_battle(&self, battle_id: BattleId) -> Result<Option<StoredBattle>, anyhow::Error> {
        let result = self.client
            .get_item()
            .table_name(&self.table_name)
            .key("battle_id", AttributeValue::S(battle_id.to_string()))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get battle: {}", e))?;

        match result.item {
            Some(item) => {
                let battle = self.item_to_battle(item)?;
                Ok(Some(battle))
            }
            None => Ok(None),
        }
    }

    /// Update an existing battle in the database
    pub async fn update_battle(&self, battle: &StoredBattle) -> Result<(), anyhow::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Serialize the battle state
        let battle_state_json = serde_json::to_string(&battle.battle_state)
            .map_err(|e| anyhow::anyhow!("Failed to serialize battle state: {}", e))?;

        self.client
            .update_item()
            .table_name(&self.table_name)
            .key("battle_id", AttributeValue::S(battle.battle_id.to_string()))
            .update_expression("SET battle_state = :state, last_updated = :timestamp")
            .expression_attribute_values(":state", AttributeValue::S(battle_state_json))
            .expression_attribute_values(":timestamp", AttributeValue::N(timestamp.to_string()))
            .condition_expression("attribute_exists(battle_id)") // Ensure battle exists
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update battle: {}", e))?;

        Ok(())
    }

    /// Delete a battle from the database (for cleanup)
    pub async fn delete_battle(&self, battle_id: BattleId) -> Result<(), anyhow::Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("battle_id", AttributeValue::S(battle_id.to_string()))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete battle: {}", e))?;

        Ok(())
    }

    /// List battles for a specific player (for potential future use)
    pub async fn list_player_battles(&self, player_id: &PlayerId) -> Result<Vec<BattleId>, anyhow::Error> {
        // Note: This would require a GSI (Global Secondary Index) on player_id fields
        // For now, this is a placeholder implementation
        todo!("Implement GSI-based player battle lookup")
    }

    /// Convert StoredBattle to DynamoDB item
    fn battle_to_item(&self, battle: &StoredBattle) -> Result<HashMap<String, AttributeValue>, anyhow::Error> {
        let mut item = HashMap::new();
        
        item.insert("battle_id".to_string(), AttributeValue::S(battle.battle_id.to_string()));
        item.insert("player1_id".to_string(), AttributeValue::S(battle.player1_id.0.clone()));
        item.insert("player2_id".to_string(), AttributeValue::S(battle.player2_id.0.clone()));
        item.insert("created_at".to_string(), AttributeValue::N(battle.created_at.to_string()));
        item.insert("last_updated".to_string(), AttributeValue::N(battle.last_updated.to_string()));

        // Serialize the battle state as JSON
        let battle_state_json = serde_json::to_string(&battle.battle_state)
            .map_err(|e| anyhow::anyhow!("Failed to serialize battle state: {}", e))?;
        item.insert("battle_state".to_string(), AttributeValue::S(battle_state_json));
        
        // Serialize the turn logs as JSON
        let turn_logs_json = serde_json::to_string(&battle.turn_logs)
            .map_err(|e| anyhow::anyhow!("Failed to serialize turn logs: {}", e))?;
        item.insert("turn_logs".to_string(), AttributeValue::S(turn_logs_json));

        Ok(item)
    }

    /// Convert DynamoDB item to StoredBattle
    fn item_to_battle(&self, item: HashMap<String, AttributeValue>) -> Result<StoredBattle, anyhow::Error> {
        let battle_id_str = item.get("battle_id")
            .and_then(|av| av.as_s().ok())
            .ok_or_else(|| anyhow::anyhow!("Missing battle_id"))?;

        let battle_id = BattleId(battle_id_str.parse()
            .map_err(|e| anyhow::anyhow!("Invalid battle_id format: {}", e))?);

        let player1_id = PlayerId(
            item.get("player1_id")
                .and_then(|av| av.as_s().ok())
                .ok_or_else(|| anyhow::anyhow!("Missing player1_id"))?
                .clone()
        );

        let player2_id = PlayerId(
            item.get("player2_id")
                .and_then(|av| av.as_s().ok())
                .ok_or_else(|| anyhow::anyhow!("Missing player2_id"))?
                .clone()
        );

        let created_at: i64 = item.get("created_at")
            .and_then(|av| av.as_n().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid created_at"))?;

        let last_updated: i64 = item.get("last_updated")
            .and_then(|av| av.as_n().ok())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid last_updated"))?;

        let battle_state_json = item.get("battle_state")
            .and_then(|av| av.as_s().ok())
            .ok_or_else(|| anyhow::anyhow!("Missing battle_state"))?;

        let battle_state = serde_json::from_str(battle_state_json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize battle state: {}", e))?;

        // Try to get turn_logs, default to empty if not found (backward compatibility)
        let turn_logs = item.get("turn_logs")
            .and_then(|v| v.as_s().ok())
            .and_then(|json| serde_json::from_str(json).ok())
            .unwrap_or_else(Vec::new); // Default to empty vec if missing or parsing fails

        Ok(StoredBattle {
            battle_id,
            player1_id,
            player2_id,
            battle_state,
            turn_logs,
            created_at,
            last_updated,
        })
    }
}