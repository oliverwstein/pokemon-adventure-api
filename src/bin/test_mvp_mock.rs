use pokemon_adventure_api::*;
use pokemon_adventure::player::PlayerAction;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Pokemon Adventure API MVP Flow (Mock Mode)");
    println!("====================================================");

    // Test 1: Get Available Teams (No DB required)
    println!("\n1️⃣ Testing Available Teams Engine Logic");
    let teams = engine::get_available_teams();
    println!("✅ Found {} teams:", teams.len());
    for team in &teams {
        println!("   - {} ({} Pokemon, avg level {})", 
                 team.name, team.pokemon_count, team.average_level);
    }

    // Test 2: Get NPC Opponents (No DB required)
    println!("\n2️⃣ Testing NPC Opponents Engine Logic");
    let opponents = engine::get_npc_opponents();
    println!("✅ Found {} opponents:", opponents.len());
    for opponent in &opponents {
        println!("   - {} ({})", opponent.name, opponent.difficulty);
    }

    // Test 3: Create Battle State (No DB required)
    println!("\n3️⃣ Testing Battle Creation Engine Logic");
    let battle_state = engine::create_mvp_battle(
        "test_battle_123".to_string(),
        "Test Trainer".to_string(),
        "venusaur_team",
        "gym_leader_easy"
    )?;
    
    println!("✅ Created battle state:");
    println!("   Game State: {:?}", battle_state.game_state);
    println!("   Turn: {}", battle_state.turn_number);
    println!("   Player: {}", battle_state.players[0].player_name);
    println!("   NPC: {}", battle_state.players[1].player_name);
    
    // Show active Pokemon
    if let Some(player_pokemon) = battle_state.players[0].active_pokemon() {
        println!("   Player's Active: {} (Level {}, HP: {}/{})", 
                 player_pokemon.name, player_pokemon.level, 
                 player_pokemon.current_hp(), player_pokemon.max_hp());
    }
    if let Some(npc_pokemon) = battle_state.players[1].active_pokemon() {
        println!("   NPC's Active: {} (Level {}, HP: {}/{})", 
                 npc_pokemon.name, npc_pokemon.level,
                 npc_pokemon.current_hp(), npc_pokemon.max_hp());
    }

    // Test 4: Get Valid Actions (No DB required)
    println!("\n4️⃣ Testing Valid Actions Engine Logic");
    let player_id = PlayerId("player_1".to_string());
    let valid_actions = engine::get_player_valid_actions(&battle_state, &player_id)?;
    println!("✅ Found {} valid actions:", valid_actions.len());
    
    // Show some example actions
    let mut move_count = 0;
    let mut switch_count = 0;
    for action in &valid_actions {
        match action {
            PlayerAction::UseMove { move_index } => {
                move_count += 1;
                if move_count <= 3 {
                    if let Some(pokemon) = battle_state.players[0].active_pokemon() {
                        if let Some(Some(move_inst)) = pokemon.moves.get(*move_index) {
                            println!("   - Use Move: {:?} (PP: {})", move_inst.move_, move_inst.pp);
                        }
                    }
                }
            }
            PlayerAction::SwitchPokemon { team_index } => {
                switch_count += 1;
                if switch_count <= 2 {
                    if let Some(Some(pokemon)) = battle_state.players[0].team.get(*team_index) {
                        println!("   - Switch to: {} (Level {})", pokemon.name, pokemon.level);
                    }
                }
            }
            PlayerAction::Forfeit => {
                println!("   - Forfeit");
            }
        }
    }
    if valid_actions.len() > 6 {
        println!("   ... and {} more actions", valid_actions.len() - 6);
    }

    // Test 5: Submit Action and Game Tick (No DB required)
    println!("\n5️⃣ Testing Action Submission and Game Tick");
    let mut current_state = battle_state;
    
    let action = PlayerAction::UseMove { move_index: 0 };
    println!("   Submitting action: {:?}", action);
    
    let (updated_state, turn_events) = engine::submit_action(
        current_state,
        &player_id,
        action
    )?;
    
    println!("✅ Action processed successfully:");
    println!("   Game State: {:?}", updated_state.game_state);
    println!("   Turn: {}", updated_state.turn_number);
    
    // Show turn events
    if !turn_events.is_empty() {
        println!("   Turn Events:");
        for event in &turn_events {
            println!("     - {}", event);
        }
    }
    
    // Show HP changes
    if let Some(player_pokemon) = updated_state.players[0].active_pokemon() {
        println!("   Player's Active: {} (HP: {}/{})", 
                 player_pokemon.name, player_pokemon.current_hp(), player_pokemon.max_hp());
    }
    if let Some(npc_pokemon) = updated_state.players[1].active_pokemon() {
        println!("   NPC's Active: {} (HP: {}/{})", 
                 npc_pokemon.name, npc_pokemon.current_hp(), npc_pokemon.max_hp());
    }

    // Test 6: Battle State View for Player
    println!("\n6️⃣ Testing Player Battle View");
    let battle_view = engine::get_battle_state_for_player(&updated_state, &player_id)?;
    println!("✅ Player battle view generated:");
    println!("   Can Act: {}", battle_view.can_act);
    println!("   Turn: {}", battle_view.turn_number);
    println!("   Opponent Pokemon Remaining: {}", battle_view.opponent_public_info.remaining_pokemon_count);
    
    if let Some(ref active) = battle_view.player_team.active_pokemon {
        println!("   Player's Active Pokemon:");
        println!("     Name: {}", active.name);
        println!("     Level: {}", active.level);
        println!("     HP: {}/{}", active.current_hp, active.max_hp);
        println!("     Moves:");
        for (i, move_opt) in active.moves.iter().enumerate() {
            if let Some(ref mv) = move_opt {
                println!("       {}: {:?} (PP: {}/{})", i+1, mv.move_, mv.pp, mv.max_pp);
            }
        }
    }

    // Test 7: Simulate Battle Progress
    println!("\n7️⃣ Simulating Battle Progress");
    current_state = updated_state;
    let mut turn_count = 1;
    
    while turn_count < 3 && !matches!(current_state.game_state, 
        pokemon_adventure::battle::state::GameState::Player1Win | 
        pokemon_adventure::battle::state::GameState::Player2Win |
        pokemon_adventure::battle::state::GameState::Draw) {
        
        let battle_view = engine::get_battle_state_for_player(&current_state, &player_id)?;
        if !battle_view.can_act {
            println!("   Player cannot act - battle may be over");
            break;
        }
        
        let valid_actions = engine::get_player_valid_actions(&current_state, &player_id)?;
        if let Some(first_action) = valid_actions.first() {
            println!("   Turn {}: Using action {:?}", turn_count + 1, first_action);
            
            let (new_state, _events) = engine::submit_action(
                current_state,
                &player_id,
                first_action.clone()
            )?;
            current_state = new_state;
            
            // Show turn results
            if let (Some(p_pok), Some(n_pok)) = (
                current_state.players[0].active_pokemon(),
                current_state.players[1].active_pokemon()
            ) {
                println!("     Player: {} ({} HP), NPC: {} ({} HP)", 
                         p_pok.name, p_pok.current_hp(),
                         n_pok.name, n_pok.current_hp());
            }
        }
        
        turn_count += 1;
    }

    println!("\n🎉 MVP Engine Logic Test Completed Successfully!");
    println!("All core engine functions working correctly:");
    println!("  ✅ get_available_teams()");
    println!("  ✅ get_npc_opponents()");
    println!("  ✅ create_mvp_battle()");
    println!("  ✅ get_player_valid_actions()");
    println!("  ✅ submit_action() with game tick loop");
    println!("  ✅ get_battle_state_for_player()");
    println!("\n🚀 Ready for AWS Lambda deployment!");
    println!("💡 To test with real HTTP requests, deploy to AWS and use the endpoints from MVP_API_TEST.md");

    Ok(())
}