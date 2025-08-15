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
        
        // Use the new constructor for the real database
        let battle_handler = BattleHandler::new_with_real_db(table_name).await
            .map_err(|e| format!("Failed to initialize battle handler: {}", e))?;

        Ok(Router { battle_handler })
    }

    pub async fn call(&self, event: lambda_runtime::LambdaEvent<Value>) -> Result<Value, Error> {
        let (payload, _context) = event.into_parts();
        
        // Extract HTTP method and path from the Lambda event (API Gateway v2 format)
        let method = payload.get("requestContext")
            .and_then(|ctx| ctx.get("http"))
            .and_then(|http| http.get("method"))
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
            
        // Extract path from rawPath and remove /prod prefix
        let raw_path = payload.get("rawPath")
            .and_then(|v| v.as_str())
            .unwrap_or("/");
            
        let path = raw_path.strip_prefix("/prod").unwrap_or(raw_path);

        info!("Processing {} {}", method, path);

        // Route the request
        let response = match (method, path) {
            // MVP Endpoints
            ("GET", "/available_teams") => self.get_available_teams().await,
            ("GET", "/npc_opponents") => self.get_npc_opponents().await, 
            ("POST", "/battles") => self.create_mvp_battle(payload).await,
            ("POST", path) if path.starts_with("/battles/") && path.ends_with("/action") => {
                self.submit_action(payload).await
            }
            ("GET", path) if path.starts_with("/battles/") && path.contains("/state") => {
                self.get_battle_state(payload).await
            }
            ("GET", path) if path.starts_with("/battles/") && path.contains("/valid_actions") => {
                self.get_valid_actions(payload).await
            }
            ("GET", path) if path.starts_with("/battles/") && path.contains("/team_info") => {
                self.get_team_info(payload).await
            }
            ("GET", path) if path.starts_with("/battles/") && path.contains("/events") => {
                self.get_battle_events(payload).await
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

    // MVP Endpoint implementations
    async fn get_available_teams(&self) -> Result<Value, anyhow::Error> {
        let response = self.battle_handler.get_available_teams().await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn get_npc_opponents(&self) -> Result<Value, anyhow::Error> {
        let response = self.battle_handler.get_npc_opponents().await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn create_mvp_battle(&self, payload: Value) -> Result<Value, anyhow::Error> {
        let body = payload.get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing request body"))?;

        let request: CreateMvpBattleRequest = serde_json::from_str(body)
            .map_err(|e| anyhow::anyhow!("Invalid request format: {}", e))?;

        let response = self.battle_handler.create_mvp_battle(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn submit_action(&self, payload: Value) -> Result<Value, anyhow::Error> {
        // Extract battle_id from path
        let raw_path = payload.get("rawPath")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let path = raw_path.strip_prefix("/prod").unwrap_or(raw_path);
        
        let battle_id_str = path
            .strip_prefix("/battles/")
            .and_then(|s| s.strip_suffix("/action"))
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

    async fn get_battle_state(&self, payload: Value) -> Result<Value, anyhow::Error> {
        let (battle_id, player_id) = self.extract_battle_and_player_from_path(payload)?;
        
        let request = GetBattleStateRequest { battle_id, player_id };
        let response = self.battle_handler.get_battle_state(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn get_valid_actions(&self, payload: Value) -> Result<Value, anyhow::Error> {
        let (battle_id, player_id) = self.extract_battle_and_player_from_path(payload)?;
        
        let request = GetValidActionsRequest { battle_id, player_id };
        let response = self.battle_handler.get_valid_actions(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn get_team_info(&self, payload: Value) -> Result<Value, anyhow::Error> {
        let (battle_id, player_id) = self.extract_battle_and_player_from_path(payload)?;
        
        let request = GetTeamInfoRequest { battle_id, player_id };
        let response = self.battle_handler.get_team_info(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    async fn get_battle_events(&self, payload: Value) -> Result<Value, anyhow::Error> {
        let (battle_id, player_id) = self.extract_battle_and_player_from_path(payload.clone())?;
        
        // Extract last_turns query parameter
        let query_params = payload.get("queryStringParameters")
            .and_then(|v| v.as_object());
        
        let last_turns = query_params
            .and_then(|params| params.get("last_turns"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u32>().ok());
        
        let request = GetBattleEventsRequest { battle_id, player_id, last_turns };
        let response = self.battle_handler.get_battle_events(request).await?;
        Ok(serde_json::to_value(response)?)
    }

    // Helper method to extract battle_id and player_id from path and query params
    fn extract_battle_and_player_from_path(&self, payload: Value) -> Result<(BattleId, PlayerId), anyhow::Error> {
        // Extract battle_id from path
        let raw_path = payload.get("rawPath")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing path"))?;
        let path = raw_path.strip_prefix("/prod").unwrap_or(raw_path);
        
        let battle_id_str = path
            .strip_prefix("/battles/")
            .and_then(|s| s.split('/').next())
            .ok_or_else(|| anyhow::anyhow!("Invalid path format"))?;

        let battle_id = BattleId(battle_id_str.parse()
            .map_err(|e| anyhow::anyhow!("Invalid battle_id: {}", e))?);

        // Extract player_id from query parameters
        let query_params = payload.get("queryStringParameters")
            .and_then(|v| v.as_object());

        let player_id = query_params
            .and_then(|params| params.get("player_id"))
            .and_then(|v| v.as_str())
            .map(|s| PlayerId(s.to_string()))
            .unwrap_or(PlayerId("player_1".to_string())); // Default to player_1 for MVP

        Ok((battle_id, player_id))
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