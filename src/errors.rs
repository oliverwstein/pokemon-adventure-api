use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::types::{BattleId, PlayerId};

/// Main API error type with HTTP status code mapping
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Battle {battle_id} not found")]
    BattleNotFound { battle_id: BattleId },

    #[error("Player {player_id} is not authorized for this battle")]
    PlayerNotAuthorized { player_id: String },

    #[error("Invalid action: {message}")]
    InvalidAction { message: String },

    #[error("Battle is in state {state:?}, cannot accept actions")]
    InvalidBattleState { state: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Internal server error: {message}")]
    InternalError { message: String },

    #[error("Authentication required")]
    AuthRequired,

    #[error("Invalid request format: {message}")]
    BadRequest { message: String },
}

impl ApiError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BattleNotFound { .. } => 404,
            ApiError::PlayerNotAuthorized { .. } => 403,
            ApiError::InvalidAction { .. } => 400,
            ApiError::InvalidBattleState { .. } => 409, // Conflict
            ApiError::DatabaseError { .. } => 500,
            ApiError::ValidationError { .. } => 400,
            ApiError::InternalError { .. } => 500,
            ApiError::AuthRequired => 401,
            ApiError::BadRequest { .. } => 400,
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BattleNotFound { .. } => "BATTLE_NOT_FOUND",
            ApiError::PlayerNotAuthorized { .. } => "PLAYER_NOT_AUTHORIZED",
            ApiError::InvalidAction { .. } => "INVALID_ACTION",
            ApiError::InvalidBattleState { .. } => "INVALID_BATTLE_STATE",
            ApiError::DatabaseError { .. } => "DATABASE_ERROR",
            ApiError::ValidationError { .. } => "VALIDATION_ERROR",
            ApiError::InternalError { .. } => "INTERNAL_ERROR",
            ApiError::AuthRequired => "AUTH_REQUIRED",
            ApiError::BadRequest { .. } => "BAD_REQUEST",
        }
    }

    /// Convert to API response format
    pub fn to_response(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            error: self.error_code().to_string(),
            message: self.to_string(),
            status_code: self.status_code(),
        }
    }
}

/// API error response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

// Convert from various error types to ApiError
impl From<aws_sdk_dynamodb::Error> for ApiError {
    fn from(err: aws_sdk_dynamodb::Error) -> Self {
        ApiError::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest {
            message: format!("JSON parsing error: {}", err),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalError {
            message: err.to_string(),
        }
    }
}

/// Helper functions for creating common errors
impl ApiError {
    pub fn battle_not_found(battle_id: BattleId) -> Self {
        ApiError::BattleNotFound { battle_id }
    }

    pub fn player_not_authorized(player_id: &PlayerId) -> Self {
        ApiError::PlayerNotAuthorized {
            player_id: player_id.0.clone(),
        }
    }

    pub fn invalid_action(message: impl Into<String>) -> Self {
        ApiError::InvalidAction {
            message: message.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        ApiError::ValidationError {
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        ApiError::BadRequest {
            message: message.into(),
        }
    }
}