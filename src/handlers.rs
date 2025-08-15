use pokemon_adventure::{
    battle::state::BattleState,
    player::PlayerAction,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::Database;
use crate::engine;
use crate::errors::ApiError;
use crate::types::*;

/// Clean architecture: Request → Router → Database (load) → Engine (logic) → Database (save) → Response
pub struct BattleHandler {
    db: Database,
}

impl BattleHandler {
    pub async fn new(table_name: String) -> Result<Self, ApiError> {
        let db = Database::new(table_name).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?;
        Ok(BattleHandler { db })
    }

    /// Create a new battle - Clean architecture implementation
    pub async fn create_battle(&self, request: CreateBattleRequest) -> Result<CreateBattleResponse, ApiError> {
        let battle_id = BattleId::new();
        
        // Engine Logic: Pure function creates battle state
        let battle_state = engine::create_battle(
            battle_id.to_string(),
            request.player1_id.clone(),
            &request.player1_team,
            request.player2_id.clone(),
            &request.player2_team,
        )?;

        // Database Save: Store the new battle
        let stored_battle = StoredBattle {
            battle_id,
            player1_id: request.player1_id,
            player2_id: request.player2_id,
            battle_state,
            turn_logs: Vec::new(), // Start with empty turn logs
            created_at: current_timestamp(),
            last_updated: current_timestamp(),
        };

        self.db.create_battle(&stored_battle).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?;

        // Response: Clean API response
        Ok(CreateBattleResponse {
            battle_id,
            status: "Battle created successfully".to_string(),
        })
    }

    /// Submit a player action - Clean architecture implementation
    pub async fn submit_action(&self, request: SubmitActionRequest) -> Result<SubmitActionResponse, ApiError> {
        // Database Load: Get current battle state
        let mut stored_battle = self.db.get_battle(request.battle_id).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?
            .ok_or_else(|| ApiError::battle_not_found(request.battle_id))?;

        // Engine Logic: Pure function processes the action
        let (new_battle_state, turn_events) = engine::submit_action(
            stored_battle.battle_state,
            &request.player_id,
            request.action,
        )?;

        // Database Save: Update battle state and turn logs
        stored_battle.battle_state = new_battle_state.clone();
        stored_battle.last_updated = current_timestamp();
        
        // Add turn log if there were events
        if !turn_events.is_empty() {
            let turn_log = TurnLog {
                turn_number: new_battle_state.turn_number,
                events: turn_events,
                timestamp: current_timestamp(),
            };
            stored_battle.turn_logs.push(turn_log);
        }
        
        self.db.update_battle(&stored_battle).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?;

        // Response: Success response
        Ok(SubmitActionResponse {
            success: true,
            message: "Action processed successfully".to_string(),
            battle_updated: true,
        })
    }

    /// Get current battle state - Clean architecture implementation
    pub async fn get_battle_state(&self, request: GetBattleStateRequest) -> Result<GetBattleStateResponse, ApiError> {
        // Database Load: Get current battle state
        let stored_battle = self.db.get_battle(request.battle_id).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?
            .ok_or_else(|| ApiError::battle_not_found(request.battle_id))?;

        // Engine Logic: Pure function creates player-specific view
        let battle_view = engine::get_battle_state_for_player(
            &stored_battle.battle_state,
            &request.player_id,
        )?;

        // Response: Convert engine view to API response
        Ok(GetBattleStateResponse {
            battle_id: request.battle_id,
            game_state: battle_view.game_state,
            turn_number: battle_view.turn_number,
            can_act: battle_view.can_act,
            player_team: convert_team_view(battle_view.player_team),
            opponent_info: convert_opponent_view(battle_view.opponent_public_info),
        })
    }

    /// Get valid actions for a player - Clean architecture implementation  
    pub async fn get_valid_actions(&self, request: GetValidActionsRequest) -> Result<GetValidActionsResponse, ApiError> {
        // Database Load: Get current battle state
        let stored_battle = self.db.get_battle(request.battle_id).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?
            .ok_or_else(|| ApiError::battle_not_found(request.battle_id))?;

        // Engine Logic: Pure function gets valid actions
        let valid_actions = engine::get_player_valid_actions(
            &stored_battle.battle_state,
            &request.player_id,
        )?;

        // Response: Convert to API format
        Ok(GetValidActionsResponse {
            battle_id: request.battle_id,
            valid_actions,
        })
    }

    /// Get team information - Clean architecture implementation
    pub async fn get_team_info(&self, request: GetTeamInfoRequest) -> Result<GetTeamInfoResponse, ApiError> {
        // Database Load: Get current battle state
        let stored_battle = self.db.get_battle(request.battle_id).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?
            .ok_or_else(|| ApiError::battle_not_found(request.battle_id))?;

        // Engine Logic: Validate player and get team view
        let battle_view = engine::get_battle_state_for_player(
            &stored_battle.battle_state,
            &request.player_id,
        )?;

        // Response: Return detailed team information
        Ok(GetTeamInfoResponse {
            battle_id: request.battle_id,
            team: convert_team_view(battle_view.player_team),
        })
    }

    /// MVP Endpoints - Get available teams
    pub async fn get_available_teams(&self) -> Result<AvailableTeamsResponse, ApiError> {
        // Engine Logic: Pure function gets prefab teams
        let teams = engine::get_available_teams();
        
        // Response: Return available teams
        Ok(AvailableTeamsResponse { teams })
    }

    /// MVP Endpoints - Get NPC opponents
    pub async fn get_npc_opponents(&self) -> Result<NpcOpponentsResponse, ApiError> {
        // Engine Logic: Pure function gets NPC opponents
        let opponents = engine::get_npc_opponents();
        
        // Response: Return NPC opponents
        Ok(NpcOpponentsResponse { opponents })
    }

    /// MVP Endpoints - Create battle with prefab team vs NPC
    pub async fn create_mvp_battle(&self, request: CreateMvpBattleRequest) -> Result<CreateMvpBattleResponse, ApiError> {
        let battle_id = BattleId::new();
        
        // Engine Logic: Create battle between player and NPC
        let battle_state = engine::create_mvp_battle(
            battle_id.to_string(),
            request.player_name.clone(),
            &request.team_id,
            &request.opponent_id,
        )?;

        // Database Save: Store the new battle
        let stored_battle = StoredBattle {
            battle_id,
            player1_id: PlayerId("player_1".to_string()),
            player2_id: PlayerId("npc".to_string()),
            battle_state: battle_state.clone(),
            turn_logs: Vec::new(), // Start with empty turn logs
            created_at: current_timestamp(),
            last_updated: current_timestamp(),
        };

        self.db.create_battle(&stored_battle).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?;

        // Response: Return battle info with initial state
        let battle_view = engine::get_battle_state_for_player(
            &battle_state,
            &PlayerId("player_1".to_string()),
        )?;

        let initial_state = GetBattleStateResponse {
            battle_id,
            game_state: battle_view.game_state,
            turn_number: battle_view.turn_number,
            can_act: battle_view.can_act,
            player_team: convert_team_view(battle_view.player_team),
            opponent_info: convert_opponent_view(battle_view.opponent_public_info),
        };

        Ok(CreateMvpBattleResponse {
            battle_id,
            status: "Battle created successfully".to_string(),
            battle_state: initial_state,
        })
    }

    /// Get battle events/log - Clean architecture implementation
    pub async fn get_battle_events(&self, request: GetBattleEventsRequest) -> Result<GetBattleEventsResponse, ApiError> {
        // Database Load: Get battle
        let stored_battle = self.db.get_battle(request.battle_id).await
            .map_err(|e| ApiError::DatabaseError { message: e.to_string() })?
            .ok_or_else(|| ApiError::battle_not_found(request.battle_id))?;

        // Validate player authorization
        let _player_index = engine::validate_player_authorization(
            &stored_battle.battle_state,
            &request.player_id,
        )?;

        // Filter turn logs based on request
        let turn_logs = if let Some(last_turns) = request.last_turns {
            // Get only the last X turns
            let total_turns = stored_battle.turn_logs.len();
            let start_index = if total_turns > last_turns as usize {
                total_turns - last_turns as usize
            } else {
                0
            };
            stored_battle.turn_logs[start_index..].to_vec()
        } else {
            // Get all turn logs
            stored_battle.turn_logs.clone()
        };

        // Response: Return filtered turn logs
        Ok(GetBattleEventsResponse {
            battle_id: request.battle_id,
            turn_logs,
            total_turns: stored_battle.battle_state.turn_number,
        })
    }
}

// Helper functions for converting engine types to API types

fn convert_team_view(team_view: engine::TeamView) -> ApiTeamView {
    ApiTeamView {
        active_pokemon: team_view.active_pokemon.map(convert_pokemon_detail),
        team_pokemon: team_view.team_pokemon.into_iter()
            .map(|p| p.map(convert_pokemon_summary))
            .collect(),
    }
}

fn convert_pokemon_detail(pokemon: engine::PokemonDetailView) -> ApiPokemonDetail {
    ApiPokemonDetail {
        name: pokemon.name,
        species: pokemon.species,
        level: pokemon.level,
        current_hp: pokemon.current_hp,
        max_hp: pokemon.max_hp,
        attack: pokemon.stats.attack,
        defense: pokemon.stats.defense,
        sp_attack: pokemon.stats.sp_attack,
        sp_defense: pokemon.stats.sp_defense,
        speed: pokemon.stats.speed,
        moves: pokemon.moves.into_iter()
            .map(|m| m.map(convert_move_view))
            .collect(),
        status: pokemon.status.map(|s| format!("{:?}", s)),
    }
}

fn convert_pokemon_summary(pokemon: engine::PokemonSummaryView) -> ApiPokemonSummary {
    ApiPokemonSummary {
        name: pokemon.name,
        species: pokemon.species,
        level: pokemon.level,
        current_hp: pokemon.current_hp,
        max_hp: pokemon.max_hp,
        is_fainted: pokemon.is_fainted,
        status: pokemon.status.map(|s| format!("{:?}", s)),
    }
}

fn convert_move_view(move_view: engine::MoveView) -> ApiMoveView {
    ApiMoveView {
        move_: move_view.move_,
        pp: move_view.pp,
        max_pp: move_view.max_pp,
    }
}

fn convert_opponent_view(opponent: engine::OpponentView) -> ApiOpponentView {
    ApiOpponentView {
        player_name: opponent.player_name,
        active_pokemon: opponent.active_pokemon.map(convert_pokemon_summary),
        remaining_pokemon_count: opponent.remaining_pokemon_count,
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}