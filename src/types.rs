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
    pub turn_logs: Vec<TurnLog>, // Events per turn for battle log
    pub created_at: i64, // Unix timestamp
    pub last_updated: i64, // Unix timestamp
}

/// Turn log entry storing events for a specific turn
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurnLog {
    pub turn_number: u32,
    pub events: Vec<String>, // Human-readable event messages
    pub timestamp: i64, // When this turn was processed
}

/// New API request/response types for clean architecture

/// Request to get battle state
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBattleStateRequest {
    pub battle_id: BattleId,
    pub player_id: PlayerId,
}

/// Response containing battle state
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBattleStateResponse {
    pub battle_id: BattleId,
    pub game_state: GameState,
    pub turn_number: u32,
    pub can_act: bool,
    pub player_team: ApiTeamView,
    pub opponent_info: ApiOpponentView,
}

/// Request to get valid actions
#[derive(Debug, Serialize, Deserialize)]
pub struct GetValidActionsRequest {
    pub battle_id: BattleId,
    pub player_id: PlayerId,
}

/// Response containing valid actions
#[derive(Debug, Serialize, Deserialize)]
pub struct GetValidActionsResponse {
    pub battle_id: BattleId,
    pub valid_actions: Vec<PlayerAction>,
}

/// Request to get team information
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTeamInfoRequest {
    pub battle_id: BattleId,
    pub player_id: PlayerId,
}

/// Response containing team information
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTeamInfoResponse {
    pub battle_id: BattleId,
    pub team: ApiTeamView,
}

/// API representation of team view
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTeamView {
    pub active_pokemon: Option<ApiPokemonDetail>,
    pub team_pokemon: Vec<Option<ApiPokemonSummary>>,
}

/// Detailed Pokemon information for API
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiPokemonDetail {
    pub name: String,
    pub species: Species,
    pub level: u8,
    pub current_hp: u16,
    pub max_hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub sp_attack: u16,
    pub sp_defense: u16,
    pub speed: u16,
    pub moves: Vec<Option<ApiMoveView>>,
    pub status: Option<String>,
}

/// Summary Pokemon information for API
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiPokemonSummary {
    pub name: String,
    pub species: Species,
    pub level: u8,
    pub current_hp: u16,
    pub max_hp: u16,
    pub is_fainted: bool,
    pub status: Option<String>,
}

/// Move information for API
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiMoveView {
    pub move_: Move,
    pub pp: u8,
    pub max_pp: u8,
}

/// Opponent information for API
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiOpponentView {
    pub player_name: String,
    pub active_pokemon: Option<ApiPokemonSummary>,
    pub remaining_pokemon_count: usize,
}

/// API types for MVP endpoints

/// Response for available teams endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableTeamsResponse {
    pub teams: Vec<PrefabTeamInfo>,
}

/// Prefab team information for API
#[derive(Debug, Serialize, Deserialize)]
pub struct PrefabTeamInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pokemon_count: usize,
    pub average_level: u8,
}

/// Response for NPC opponents endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct NpcOpponentsResponse {
    pub opponents: Vec<NpcOpponentInfo>,
}

/// NPC opponent information for API
#[derive(Debug, Serialize, Deserialize)]
pub struct NpcOpponentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: String,
}

/// MVP Create battle request (simplified)
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMvpBattleRequest {
    pub player_name: String,
    pub team_id: String,
    pub opponent_id: String,
}

/// MVP Create battle response
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMvpBattleResponse {
    pub battle_id: BattleId,
    pub status: String,
    pub battle_state: GetBattleStateResponse, // Include initial state
}

/// Request to get battle events/log
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBattleEventsRequest {
    pub battle_id: BattleId,
    pub player_id: PlayerId,
    pub last_turns: Option<u32>, // If specified, get only the last X turns; if None, get all
}

/// Response containing battle events
#[derive(Debug, Serialize, Deserialize)]
pub struct GetBattleEventsResponse {
    pub battle_id: BattleId,
    pub turn_logs: Vec<TurnLog>,
    pub total_turns: u32,
}