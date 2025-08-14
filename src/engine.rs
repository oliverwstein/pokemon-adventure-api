use pokemon_adventure::{
    battle::{
        state::{BattleState, GameState, TurnRng},
        engine::{
            collect_player_actions, resolve_turn, ready_for_turn_resolution,
            get_valid_actions, validate_player_action,
        },
    },
    player::{BattlePlayer, PlayerAction},
    pokemon::{PokemonInst, get_species_data},
    species::Species,
    moves::Move,
};
use crate::errors::ApiError;
use crate::types::{TeamPokemon, PlayerId};

/// Pure engine functions - no I/O dependencies, just game logic

/// Create a new battle state from team configurations
pub fn create_battle(
    battle_id: String,
    player1_id: PlayerId,
    player1_team: &[TeamPokemon],
    player2_id: PlayerId,
    player2_team: &[TeamPokemon],
) -> Result<BattleState, ApiError> {
    // Validate and create teams
    let team1 = create_pokemon_team(player1_team)?;
    let team2 = create_pokemon_team(player2_team)?;

    // Create battle players
    let player1 = BattlePlayer::new(
        player1_id.0.clone(),
        format!("Player {}", player1_id.0),
        team1,
    );

    let player2 = BattlePlayer::new(
        player2_id.0.clone(),
        format!("Player {}", player2_id.0),
        team2,
    );

    // Initialize battle state
    let battle_state = BattleState::new(battle_id, player1, player2);
    Ok(battle_state)
}

/// Submit a player action and process the battle forward
/// This implements the "game tick" loop from the API plan
pub fn submit_action(
    mut battle_state: BattleState,
    player_id: &PlayerId,
    action: PlayerAction,
) -> Result<BattleState, ApiError> {
    // Determine which player is acting
    let player_index = get_player_index(&battle_state, player_id)?;

    // Validate the action is legal in current game state
    validate_action_context(&battle_state, player_index, &action)?;

    // Validate the specific action details
    validate_player_action(&battle_state, player_index, &action)
        .map_err(|e| ApiError::invalid_action(e))?;

    // Apply the action to the battle state
    battle_state.action_queue[player_index] = Some(action);

    // Process battle forward as far as possible ("game tick" loop)
    process_battle_ticks(&mut battle_state)?;

    Ok(battle_state)
}

/// Get all valid actions for a player
pub fn get_player_valid_actions(
    battle_state: &BattleState,
    player_id: &PlayerId,
) -> Result<Vec<PlayerAction>, ApiError> {
    let player_index = get_player_index(battle_state, player_id)?;
    Ok(get_valid_actions(battle_state, player_index))
}

/// Validate that a player is authorized to act on behalf of this battle
pub fn validate_player_authorization(
    battle_state: &BattleState,
    player_id: &PlayerId,
) -> Result<usize, ApiError> {
    get_player_index(battle_state, player_id)
}

/// Get current battle state information for a specific player
pub fn get_battle_state_for_player(
    battle_state: &BattleState,
    requesting_player_id: &PlayerId,
) -> Result<PlayerBattleView, ApiError> {
    let player_index = get_player_index(battle_state, requesting_player_id)?;
    let opponent_index = 1 - player_index;

    let player = &battle_state.players[player_index];
    let opponent = &battle_state.players[opponent_index];

    Ok(PlayerBattleView {
        game_state: battle_state.game_state,
        turn_number: battle_state.turn_number,
        player_team: create_player_team_view(player),
        opponent_public_info: create_opponent_view(opponent),
        can_act: can_player_act(battle_state, player_index),
    })
}

/// Data structure for battle state from a player's perspective
#[derive(Debug)]
pub struct PlayerBattleView {
    pub game_state: GameState,
    pub turn_number: u32,
    pub player_team: TeamView,
    pub opponent_public_info: OpponentView,
    pub can_act: bool,
}

#[derive(Debug)]
pub struct TeamView {
    pub active_pokemon: Option<PokemonDetailView>,
    pub team_pokemon: Vec<Option<PokemonSummaryView>>,
}

#[derive(Debug)]
pub struct PokemonDetailView {
    pub name: String,
    pub species: Species,
    pub level: u8,
    pub current_hp: u16,
    pub max_hp: u16,
    pub stats: pokemon_adventure::pokemon::CurrentStats,
    pub moves: Vec<Option<MoveView>>,
    pub status: Option<pokemon_adventure::pokemon::StatusCondition>,
}

#[derive(Debug)]
pub struct PokemonSummaryView {
    pub name: String,
    pub species: Species,
    pub level: u8,
    pub current_hp: u16,
    pub max_hp: u16,
    pub is_fainted: bool,
    pub status: Option<pokemon_adventure::pokemon::StatusCondition>,
}

#[derive(Debug)]
pub struct MoveView {
    pub move_: Move,
    pub pp: u8,
    pub max_pp: u8,
}

#[derive(Debug)]
pub struct OpponentView {
    pub player_name: String,
    pub active_pokemon: Option<PokemonSummaryView>,
    pub remaining_pokemon_count: usize,
}

// Helper functions

fn create_pokemon_team(team_config: &[TeamPokemon]) -> Result<Vec<PokemonInst>, ApiError> {
    if team_config.is_empty() {
        return Err(ApiError::validation_error("Team cannot be empty"));
    }

    if team_config.len() > 6 {
        return Err(ApiError::validation_error("Team cannot have more than 6 Pokemon"));
    }

    let mut pokemon_team = Vec::new();

    for team_pokemon in team_config {
        // Validate level range
        if team_pokemon.level == 0 || team_pokemon.level > 100 {
            return Err(ApiError::validation_error(
                format!("Invalid level {} for {:?}", team_pokemon.level, team_pokemon.species)
            ));
        }

        // Get species data
        let species_data = get_species_data(team_pokemon.species)
            .ok_or_else(|| ApiError::validation_error(
                format!("Species data not found for {:?}", team_pokemon.species)
            ))?;

        // Validate moves
        if team_pokemon.moves.is_empty() || team_pokemon.moves.len() > 4 {
            return Err(ApiError::validation_error(
                "Pokemon must have 1-4 moves"
            ));
        }

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

    Ok(pokemon_team)
}

fn get_player_index(battle_state: &BattleState, player_id: &PlayerId) -> Result<usize, ApiError> {
    if battle_state.players[0].player_id == player_id.0 {
        Ok(0)
    } else if battle_state.players[1].player_id == player_id.0 {
        Ok(1)
    } else {
        Err(ApiError::player_not_authorized(player_id))
    }
}

fn validate_action_context(
    battle_state: &BattleState,
    player_index: usize,
    action: &PlayerAction,
) -> Result<(), ApiError> {
    match battle_state.game_state {
        GameState::WaitingForActions => {
            // Normal turn - validate player can act
            if battle_state.action_queue[player_index].is_some() {
                return Err(ApiError::invalid_action("Player has already submitted an action for this turn"));
            }
        }
        GameState::WaitingForPlayer1Replacement => {
            if player_index != 0 {
                return Err(ApiError::invalid_action("Only Player 1 can act during replacement phase"));
            }
            if !matches!(action, PlayerAction::SwitchPokemon { .. }) {
                return Err(ApiError::invalid_action("Must switch Pokemon during replacement phase"));
            }
        }
        GameState::WaitingForPlayer2Replacement => {
            if player_index != 1 {
                return Err(ApiError::invalid_action("Only Player 2 can act during replacement phase"));
            }
            if !matches!(action, PlayerAction::SwitchPokemon { .. }) {
                return Err(ApiError::invalid_action("Must switch Pokemon during replacement phase"));
            }
        }
        GameState::WaitingForBothReplacements => {
            if !matches!(action, PlayerAction::SwitchPokemon { .. }) {
                return Err(ApiError::invalid_action("Must switch Pokemon during replacement phase"));
            }
        }
        _ => {
            return Err(ApiError::InvalidBattleState {
                state: format!("{:?}", battle_state.game_state),
            });
        }
    }

    Ok(())
}

fn process_battle_ticks(battle_state: &mut BattleState) -> Result<(), ApiError> {
    // Collect AI actions as needed
    collect_player_actions(battle_state)
        .map_err(|e| ApiError::InternalError { message: e })?;

    // Process battle forward as far as possible
    let mut iterations = 0;
    const MAX_ITERATIONS: u32 = 100; // Prevent infinite loops

    while ready_for_turn_resolution(battle_state) && iterations < MAX_ITERATIONS {
        let rng = TurnRng::new_random();
        let _event_bus = resolve_turn(battle_state, rng);

        // Check if battle ended
        match battle_state.game_state {
            GameState::Player1Win | GameState::Player2Win | GameState::Draw => {
                break;
            }
            _ => {}
        }

        iterations += 1;
    }

    if iterations >= MAX_ITERATIONS {
        return Err(ApiError::InternalError {
            message: "Battle processing exceeded maximum iterations".to_string(),
        });
    }

    Ok(())
}

fn can_player_act(battle_state: &BattleState, player_index: usize) -> bool {
    match battle_state.game_state {
        GameState::WaitingForActions => battle_state.action_queue[player_index].is_none(),
        GameState::WaitingForPlayer1Replacement => player_index == 0,
        GameState::WaitingForPlayer2Replacement => player_index == 1,
        GameState::WaitingForBothReplacements => true,
        _ => false,
    }
}

fn create_player_team_view(player: &BattlePlayer) -> TeamView {
    let active_pokemon = player.active_pokemon().map(|pokemon| PokemonDetailView {
        name: pokemon.name.clone(),
        species: pokemon.species,
        level: pokemon.level,
        current_hp: pokemon.current_hp(),
        max_hp: pokemon.max_hp(),
        stats: pokemon.stats.clone(),
        moves: pokemon.moves.iter().map(|move_slot| {
            move_slot.as_ref().map(|move_inst| MoveView {
                move_: move_inst.move_,
                pp: move_inst.pp,
                max_pp: move_inst.max_pp(),
            })
        }).collect(),
        status: pokemon.status,
    });

    let team_pokemon = player.team.iter().map(|pokemon_slot| {
        pokemon_slot.as_ref().map(|pokemon| PokemonSummaryView {
            name: pokemon.name.clone(),
            species: pokemon.species,
            level: pokemon.level,
            current_hp: pokemon.current_hp(),
            max_hp: pokemon.max_hp(),
            is_fainted: pokemon.is_fainted(),
            status: pokemon.status,
        })
    }).collect();

    TeamView {
        active_pokemon,
        team_pokemon,
    }
}

fn create_opponent_view(opponent: &BattlePlayer) -> OpponentView {
    let active_pokemon = opponent.active_pokemon().map(|pokemon| PokemonSummaryView {
        name: pokemon.name.clone(),
        species: pokemon.species,
        level: pokemon.level,
        current_hp: pokemon.current_hp(),
        max_hp: pokemon.max_hp(),
        is_fainted: pokemon.is_fainted(),
        status: pokemon.status,
    });

    let remaining_pokemon_count = opponent.team.iter()
        .filter_map(|p| p.as_ref())
        .filter(|p| !p.is_fainted())
        .count();

    OpponentView {
        player_name: opponent.player_name.clone(),
        active_pokemon,
        remaining_pokemon_count,
    }
}