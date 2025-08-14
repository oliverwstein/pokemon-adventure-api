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
        let new_battle_state = engine::submit_action(
            stored_battle.battle_state,
            &request.player_id,
            request.action,
        )?;

        // Database Save: Update battle state
        stored_battle.battle_state = new_battle_state;
        stored_battle.last_updated = current_timestamp();
        
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