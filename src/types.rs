use pokemon_adventure::{
    battle::state::{BattleState, GameState},
    moves::Move,
    player::PlayerAction,
    species::Species,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a battle
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattleId(pub Uuid);

impl BattleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for BattleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Player identifier  
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlayerId(pub String);

/// Request to create a new battle
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBattleRequest {
    pub player1_id: PlayerId,
    pub player2_id: PlayerId,
    pub player1_team: Vec<TeamPokemon>,
    pub player2_team: Vec<TeamPokemon>,
}

/// Pokemon configuration for team setup
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamPokemon {
    pub species: Species,
    pub level: u8,
    pub moves: Vec<Move>,
    pub nickname: Option<String>,
}

/// Response when creating a battle
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBattleResponse {
    pub battle_id: BattleId,
    pub status: String,
}

/// Request to submit a player action
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitActionRequest {
    pub battle_id: BattleId,
    pub player_id: PlayerId,
    pub action: PlayerAction,
}

/// Response after submitting an action
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitActionResponse {
    pub success: bool,
    pub message: String,
    pub battle_updated: bool,
}

/// Request to get current battle state
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBattleRequest {
    pub battle_id: BattleId,
    pub player_id: Option<PlayerId>, // For player-specific views
}

/// Response containing battle state information
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBattleResponse {
    pub battle_id: BattleId,
    pub game_state: GameState,
    pub current_turn: u32,
    pub players: Vec<PlayerSummary>,
    pub events: Vec<String>, // Recent battle events as human-readable strings
}

/// Summary information about a player in the battle
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSummary {
    pub player_id: PlayerId,
    pub player_name: String,
    pub active_pokemon: Option<PokemonSummary>,
    pub team_size: usize,
    pub remaining_pokemon: usize,
}

/// Summary information about a Pokemon
#[derive(Debug, Serialize, Deserialize)]
pub struct PokemonSummary {
    pub name: String,
    pub species: Species,
    pub level: u8,
    pub current_hp: u16,
    pub max_hp: u16,
    pub status: Option<String>,
}

/// Stored battle data in DynamoDB
#[derive(Debug, Serialize, Deserialize)]
pub struct StoredBattle {
    pub battle_id: BattleId,
    pub player1_id: PlayerId,
    pub player2_id: PlayerId,
    pub battle_state: BattleState,
    pub created_at: i64, // Unix timestamp
    pub last_updated: i64, // Unix timestamp
}

/// API Error types
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl ApiError {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
        }
    }
    
    pub fn battle_not_found(battle_id: BattleId) -> Self {
        Self::new("BattleNotFound", &format!("Battle {} not found", battle_id))
    }
    
    pub fn invalid_player(player_id: &PlayerId) -> Self {
        Self::new("InvalidPlayer", &format!("Player {} is not part of this battle", player_id.0))
    }
    
    pub fn invalid_action(message: &str) -> Self {
        Self::new("InvalidAction", message)
    }
}