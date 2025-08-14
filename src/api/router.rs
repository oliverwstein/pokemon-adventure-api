use lambda_runtime::Error;
use serde_json::{json, Value};
use tracing::{info, error};

use crate::errors::ApiError;
use crate::handlers::BattleHandler;
use crate::types::*;

pub struct Router {
    battle_handler: BattleHandler,
}

impl Router {
    pub async fn new() -> Result<Self, Error> {
        let table_name = std::env::var("DYNAMODB_TABLE_NAME")
            .unwrap_or_else(|_| "pokemon-battles".to_string());
        
        let battle_handler = BattleHandler::new(table_name).await
            .map_err(|e| format!("Failed to initialize battle handler: {}", e))?;

        Ok(Router { battle_handler })
    }

    pub async fn call(&self, event: lambda_runtime::LambdaEvent<Value>) -> Result<Value, Error> {
        let (payload, _context) = event.into_parts();
        
        // Extract HTTP method and path from the Lambda event
        let method = payload.get("httpMethod")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
            
        let path = payload.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/");

        info!("Processing {} {}", method, path);

        // Route the request
        let response = match (method, path) {
            ("POST", "/battles") => self.create_battle(payload).await,
            ("POST", path) if path.starts_with("/battles/") && path.ends_with("/actions") => {
                self.submit_action(payload).await
            }
            ("GET", path) if path.starts_with("/battles/") => {
                self.get_battle(payload).await
            }
            ("GET", "/health") => Ok(json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
            _ => Ok(self.not_found()),
        };

        match response {
            Ok(body) => Ok(json!({
                "statusCode": 200,
                "headers": {
                    "Content-Type": "application/json",
                    "Access-Control-Allow-Origin": "*",
                    "Access-Control-Allow-Methods": "GET,POST,PUT,DELETE,OPTIONS",
                    "Access-Control-Allow-Headers": "Content-Type,Authorization"
                },
                "body": serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string())
            })),
            Err(e) => {
                error!("API Error: {}", e);
                Ok(json!({
                    "statusCode": 500,
                    "headers": {
                        "Content-Type": "application/json"
                    },
                    "body": serde_json::to_string(&ApiError::InternalError { message: e.to_string() }.to_response())
                        .unwrap_or_else(|_| r#"{"error":"InternalError","message":"Unknown error"}"#.to_string())
                }))
            }
        }
    }

    async fn create_battle(&self, payload: Value) -> Result<Value, anyhow::Error> {
        let body = payload.get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing request body"))?;

        let request: CreateBattleRequest = serde_json::from_str(body)
            .map_err(|e| anyhow::anyhow!("Invalid request format: {}", e))?;

        let response = self.battle_handler.create_battle(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn submit_action(&self, payload: Value) -> Result<Value, anyhow::Error> {
        // Extract battle_id from path
        let path = payload.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        
        let battle_id_str = path
            .strip_prefix("/battles/")
            .and_then(|s| s.strip_suffix("/actions"))
            .ok_or_else(|| anyhow::anyhow!("Invalid path format"))?;

        let battle_id = BattleId(battle_id_str.parse()
            .map_err(|e| anyhow::anyhow!("Invalid battle_id: {}", e))?);

        let body = payload.get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing request body"))?;

        let mut action_request: SubmitActionRequest = serde_json::from_str(body)
            .map_err(|e| anyhow::anyhow!("Invalid request format: {}", e))?;

        // Override battle_id from URL
        action_request.battle_id = battle_id;

        let response = self.battle_handler.submit_action(action_request).await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn get_battle(&self, payload: Value) -> Result<Value, anyhow::Error> {
        // Extract battle_id from path
        let path = payload.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        
        let battle_id_str = path
            .strip_prefix("/battles/")
            .ok_or_else(|| anyhow::anyhow!("Invalid path format"))?;

        let battle_id = BattleId(battle_id_str.parse()
            .map_err(|e| anyhow::anyhow!("Invalid battle_id: {}", e))?);

        // Extract player_id from query parameters if present
        let query_params = payload.get("queryStringParameters")
            .and_then(|v| v.as_object());

        let player_id = query_params
            .and_then(|params| params.get("player_id"))
            .and_then(|v| v.as_str())
            .map(|s| PlayerId(s.to_string()));

        // For now, use get_battle_state as the basic implementation
        let battle_state_request = GetBattleStateRequest {
            battle_id,
            player_id: player_id.unwrap_or(PlayerId("system".to_string())),
        };
        let response = self.battle_handler.get_battle_state(battle_state_request).await?;
        Ok(serde_json::to_value(response)?)
    }

    fn not_found(&self) -> Value {
        json!({
            "statusCode": 404,
            "headers": {
                "Content-Type": "application/json"
            },
            "body": serde_json::to_string(&ApiError::BadRequest { message: "Endpoint not found".to_string() }.to_response())
                .unwrap_or_else(|_| r#"{"error":"NotFound","message":"Endpoint not found"}"#.to_string())
        })
    }
}

pub async fn create_router() -> Result<Router, Error> {
    Router::new().await
}