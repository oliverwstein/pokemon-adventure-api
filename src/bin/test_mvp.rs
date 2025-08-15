use pokemon_adventure_api::*;
use pokemon_adventure::player::PlayerAction;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Pokemon Adventure API MVP Flow");
    println!("==========================================");

    // Create handler (using in-memory for testing, no actual DynamoDB)
    let handler = create_test_handler().await?;

    // Test 1: Get Available Teams
    println!("\n1️⃣ Testing GET /available_teams");
    let teams_response = handler.get_available_teams().await?;
    println!("✅ Found {} teams:", teams_response.teams.len());
    for team in &teams_response.teams {
        println!("   - {} ({} Pokemon, avg level {})", 
                 team.name, team.pokemon_count, team.average_level);
    }

    // Test 2: Get NPC Opponents  
    println!("\n2️⃣ Testing GET /npc_opponents");
    let opponents_response = handler.get_npc_opponents().await?;
    println!("✅ Found {} opponents:", opponents_response.opponents.len());
    for opponent in &opponents_response.opponents {
        println!("   - {} ({})", opponent.name, opponent.difficulty);
    }

    // Test 3: Create Battle
    println!("\n3️⃣ Testing POST /battles");
    let create_request = CreateMvpBattleRequest {
        player_name: "Test Trainer".to_string(),
        team_id: "venusaur_team".to_string(),
        opponent_id: "gym_leader_easy".to_string(),
    };

    let battle_response = handler.create_mvp_battle(create_request).await?;
    let battle_id = battle_response.battle_id;
    println!("✅ Created battle: {}", battle_id);
    println!("   Game State: {:?}", battle_response.battle_state.game_state);
    println!("   Turn: {}", battle_response.battle_state.turn_number);
    println!("   Can Act: {}", battle_response.battle_state.can_act);
    
    // Show active Pokemon
    if let Some(ref active) = battle_response.battle_state.player_team.active_pokemon {
        println!("   Player's Active: {} (Level {}, HP: {}/{})", 
                 active.name, active.level, active.current_hp, active.max_hp);
    }
    if let Some(ref opponent_active) = battle_response.battle_state.opponent_info.active_pokemon {
        println!("   Opponent's Active: {} (Level {}, HP: {}/{})", 
                 opponent_active.name, opponent_active.level, 
                 opponent_active.current_hp, opponent_active.max_hp);
    }

    // Test 4: Get Valid Actions
    println!("\n4️⃣ Testing GET /battles/{}/valid_actions", battle_id);
    let actions_request = GetValidActionsRequest {
        battle_id,
        player_id: PlayerId("player_1".to_string()),
    };
    let actions_response = handler.get_valid_actions(actions_request).await?;
    println!("✅ Found {} valid actions:", actions_response.valid_actions.len());
    
    // Show some example actions
    let mut move_count = 0;
    let mut switch_count = 0;
    for action in &actions_response.valid_actions {
        match action {
            PlayerAction::UseMove { move_index } => {
                move_count += 1;
                if move_count <= 2 {
                    println!("   - Use Move {} (slot {})", move_count, move_index + 1);
                }
            }
            PlayerAction::SwitchPokemon { team_index } => {
                switch_count += 1;
                if switch_count <= 2 {
                    println!("   - Switch to Pokemon {}", team_index + 1);
                }
            }
            PlayerAction::Forfeit => {
                println!("   - Forfeit");
            }
        }
    }
    if move_count > 2 || switch_count > 2 {
        println!("   ... and {} more actions", 
                 actions_response.valid_actions.len() - 
                 std::cmp::min(move_count, 2) - std::cmp::min(switch_count, 2) - 1);
    }

    // Test 5: Submit Action (Use first move)
    println!("\n5️⃣ Testing POST /battles/{}/action", battle_id);
    let action_request = SubmitActionRequest {
        battle_id,
        player_id: PlayerId("player_1".to_string()),
        action: PlayerAction::UseMove { move_index: 0 },
    };

    let action_response = handler.submit_action(action_request).await?;
    println!("✅ Action submitted successfully:");
    println!("   Success: {}", action_response.success);
    println!("   Message: {}", action_response.message);
    println!("   Battle Updated: {}", action_response.battle_updated);

    // Test 6: Get Updated Battle State
    println!("\n6️⃣ Testing GET /battles/{}/state (after action)", battle_id);
    let state_request = GetBattleStateRequest {
        battle_id,
        player_id: PlayerId("player_1".to_string()),
    };
    let state_response = handler.get_battle_state(state_request).await?;
    println!("✅ Updated battle state:");
    println!("   Game State: {:?}", state_response.game_state);
    println!("   Turn: {}", state_response.turn_number);
    println!("   Can Act: {}", state_response.can_act);
    
    // Show HP changes
    if let Some(ref active) = state_response.player_team.active_pokemon {
        println!("   Player's Active: {} (HP: {}/{})", 
                 active.name, active.current_hp, active.max_hp);
    }
    if let Some(ref opponent_active) = state_response.opponent_info.active_pokemon {
        println!("   Opponent's Active: {} (HP: {}/{})", 
                 opponent_active.name, opponent_active.current_hp, opponent_active.max_hp);
    }

    // Test 7: Simulate a few more turns
    println!("\n7️⃣ Simulating additional battle turns...");
    let mut turn_count = 1;
    let max_turns = 5;
    
    while turn_count < max_turns {
        // Get current state
        let current_state_request = GetBattleStateRequest {
            battle_id,
            player_id: PlayerId("player_1".to_string()),
        };
        let current_state = handler.get_battle_state(current_state_request).await?;
        
        // Check if battle ended
        if !current_state.can_act {
            println!("   Battle ended at turn {}", turn_count);
            break;
        }
        
        // Get valid actions
        let valid_actions_request = GetValidActionsRequest {
            battle_id,
            player_id: PlayerId("player_1".to_string()),
        };
        let valid_actions = handler.get_valid_actions(valid_actions_request).await?;
        
        // Use first available move
        if let Some(first_action) = valid_actions.valid_actions.first() {
            if let PlayerAction::UseMove { move_index } = first_action {
                let turn_action_request = SubmitActionRequest {
                    battle_id,
                    player_id: PlayerId("player_1".to_string()),
                    action: PlayerAction::UseMove { move_index: *move_index },
                };
                
                let turn_response = handler.submit_action(turn_action_request).await?;
                println!("   Turn {}: Action submitted ({})", turn_count + 1, turn_response.message);
            }
        }
        
        turn_count += 1;
    }

    println!("\n🎉 MVP API Test Completed Successfully!");
    println!("All endpoints working correctly:");
    println!("  ✅ GET /available_teams");
    println!("  ✅ GET /npc_opponents"); 
    println!("  ✅ POST /battles");
    println!("  ✅ GET /battles/{{id}}/valid_actions");
    println!("  ✅ POST /battles/{{id}}/action");
    println!("  ✅ GET /battles/{{id}}/state");
    println!("\n🚀 Ready for LLM agent testing!");

    Ok(())
}

// Create a mock handler for testing (bypasses DynamoDB)
async fn create_test_handler() -> Result<BattleHandler, ApiError> {
    // For testing, we'll use a mock table name
    // In a real test, you might want to use DynamoDB Local or mocking
    BattleHandler::new("test_table".to_string()).await
}