use pokemon_adventure::{
    battle::{
        state::{BattleState, GameState}, 
        engine::{collect_player_actions, resolve_turn, ready_for_turn_resolution}
    },
    player::BattlePlayer,
    pokemon::PokemonInst,
    species::Species,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::Database;
use crate::types::*;

pub struct BattleHandler {
    db: Database,
}

impl BattleHandler {
    pub async fn new(table_name: String) -> Result<Self, anyhow::Error> {
        let db = Database::new(table_name).await?;
        Ok(BattleHandler { db })
    }

    /// Create a new battle
    pub async fn create_battle(&self, request: CreateBattleRequest) -> Result<CreateBattleResponse, anyhow::Error> {
        let battle_id = BattleId::new();
        
        // Convert API team format to engine format
        let player1_team = self.create_pokemon_team(&request.player1_team)?;
        let player2_team = self.create_pokemon_team(&request.player2_team)?;

        // Create battle players
        let player1 = BattlePlayer::new(
            request.player1_id.0.clone(),
            format!("Player {}", request.player1_id.0),
            player1_team,
        );
        
        let player2 = BattlePlayer::new(
            request.player2_id.0.clone(),
            format!("Player {}", request.player2_id.0),
            player2_team,
        );

        // Initialize battle state
        let battle_state = BattleState::new(battle_id.to_string(), player1, player2);

        // Store in database
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let stored_battle = StoredBattle {
            battle_id,
            player1_id: request.player1_id,
            player2_id: request.player2_id,
            battle_state,
            created_at: timestamp,
            last_updated: timestamp,
        };

        self.db.create_battle(&stored_battle).await?;

        Ok(CreateBattleResponse {
            battle_id,
            status: "Battle created successfully".to_string(),
        })
    }

    /// Submit a player action
    pub async fn submit_action(&self, request: SubmitActionRequest) -> Result<SubmitActionResponse, anyhow::Error> {
        // Retrieve battle from database
        let mut stored_battle = self.db.get_battle(request.battle_id).await?
            .ok_or_else(|| anyhow::anyhow!("Battle not found"))?;

        // Validate player is part of this battle
        if request.player_id != stored_battle.player1_id && request.player_id != stored_battle.player2_id {
            return Ok(SubmitActionResponse {
                success: false,
                message: "Player is not part of this battle".to_string(),
                battle_updated: false,
            });
        }

        // Determine player index (0 or 1)
        let player_index = if request.player_id == stored_battle.player1_id { 0 } else { 1 };

        // Apply the action to the battle state
        match self.apply_player_action(&mut stored_battle.battle_state, player_index, request.action) {
            Ok(battle_updated) => {
                // Save updated battle state if it changed
                if battle_updated {
                    stored_battle.last_updated = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    
                    self.db.update_battle(&stored_battle).await?;
                }

                Ok(SubmitActionResponse {
                    success: true,
                    message: "Action submitted successfully".to_string(),
                    battle_updated,
                })
            }
            Err(e) => Ok(SubmitActionResponse {
                success: false,
                message: e,
                battle_updated: false,
            })
        }
    }

    /// Get current battle state
    pub async fn get_battle(&self, request: GetBattleRequest) -> Result<GetBattleResponse, anyhow::Error> {
        let stored_battle = self.db.get_battle(request.battle_id).await?
            .ok_or_else(|| anyhow::anyhow!("Battle not found"))?;

        // Extract values to avoid borrow checking issues
        let battle_id = stored_battle.battle_id;
        let game_state = stored_battle.battle_state.game_state;
        let current_turn = stored_battle.battle_state.turn_number;
        
        // Convert battle state to API response format
        let players = vec![
            self.create_player_summary(&stored_battle.battle_state, 0, &stored_battle.player1_id),
            self.create_player_summary(&stored_battle.battle_state, 1, &stored_battle.player2_id),
        ];

        let events = self.format_recent_events(&stored_battle.battle_state);

        Ok(GetBattleResponse {
            battle_id,
            game_state,
            current_turn,
            players,
            events,
        })
    }

    /// Create Pokemon team from API request format
    fn create_pokemon_team(&self, team: &[TeamPokemon]) -> Result<Vec<PokemonInst>, anyhow::Error> {
        let mut pokemon_team = Vec::new();
        
        for team_pokemon in team {
            // Get species data using the compiled data system
            let species_data = pokemon_adventure::pokemon::get_species_data(team_pokemon.species)
                .ok_or_else(|| anyhow::anyhow!("Species data not found for {:?}", team_pokemon.species))?;

            // Create Pokemon instance
            let pokemon = PokemonInst::new(
                team_pokemon.species,
                &species_data,
                team_pokemon.level,
                None, // Use default IVs
                Some(team_pokemon.moves.clone()),
            );

            pokemon_team.push(pokemon);
        }

        if pokemon_team.is_empty() {
            return Err(anyhow::anyhow!("Team cannot be empty"));
        }

        Ok(pokemon_team)
    }

    /// Apply a player action to the battle state
    fn apply_player_action(
        &self, 
        battle_state: &mut BattleState, 
        player_index: usize, 
        action: pokemon_adventure::player::PlayerAction
    ) -> Result<bool, String> {
        // Check if battle is in a state where we can accept actions
        match battle_state.game_state {
            GameState::WaitingForActions => {
                // Set the player's action in the action queue
                battle_state.action_queue[player_index] = Some(action);

                // Check if both players have submitted actions
                let both_ready = battle_state.action_queue.iter()
                    .all(|action| action.is_some());

                if both_ready {
                    // Process the turn using the engine
                    self.process_battle_turn(battle_state)?;
                    Ok(true)
                } else {
                    Ok(true) // Action stored, waiting for other player
                }
            }
            _ => Err("Battle is not waiting for player actions".to_string())
        }
    }

    /// Process a complete battle turn using the engine
    fn process_battle_turn(&self, battle_state: &mut BattleState) -> Result<(), String> {
        // Use the engine's turn processing system
        while ready_for_turn_resolution(battle_state) {
            let rng = pokemon_adventure::battle::state::TurnRng::new_random();
            let _event_bus = resolve_turn(battle_state, rng);
            
            // Check if battle ended
            match battle_state.game_state {
                GameState::Player1Win | GameState::Player2Win | GameState::Draw => {
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    /// Create player summary for API response
    fn create_player_summary(&self, battle_state: &BattleState, player_index: usize, player_id: &PlayerId) -> PlayerSummary {
        let player = &battle_state.players[player_index];
        
        let active_pokemon = player.active_pokemon().map(|pokemon| PokemonSummary {
            name: pokemon.name.clone(),
            species: pokemon.species,
            level: pokemon.level,
            current_hp: pokemon.current_hp(),
            max_hp: pokemon.max_hp(),
            status: pokemon.status.as_ref().map(|s| format!("{:?}", s)),
        });

        let remaining_pokemon = player.team.iter()
            .filter_map(|p| p.as_ref())
            .filter(|p| !p.is_fainted())
            .count();

        PlayerSummary {
            player_id: player_id.clone(),
            player_name: player.player_name.clone(),
            active_pokemon,
            team_size: player.team.len(),
            remaining_pokemon,
        }
    }

    /// Format recent battle events for API response
    fn format_recent_events(&self, battle_state: &BattleState) -> Vec<String> {
        // For now, return basic state information
        // In a full implementation, you'd maintain an event log
        vec![
            format!("Turn {}", battle_state.turn_number),
            format!("Game state: {:?}", battle_state.game_state),
        ]
    }
}