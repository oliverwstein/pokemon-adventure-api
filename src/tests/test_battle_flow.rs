use pokemon_adventure::player::PlayerAction;
use crate::tests::common::create_test_handler;
use crate::{ApiError, BattleHandler, CreateMvpBattleRequest, GetBattleEventsRequest, GetBattleStateRequest, PlayerId, SubmitActionRequest};


#[tokio::test]
async fn test_solar_beam_two_turn_flow() {
    println!("--- Testing Solar Beam Two-Turn Flow ---");

    // 1. Setup: Create the handler and the battle (now using the common helper)
    let handler = create_test_handler().unwrap();
    let create_request = CreateMvpBattleRequest {
        player_name: "Test Trainer".to_string(),
        team_id: "venusaur_team".to_string(),
        opponent_id: "gym_leader_easy".to_string(),
    };
    let create_response = handler.create_mvp_battle(create_request).await.unwrap();
    let battle_id = create_response.battle_id;
    let player_id = PlayerId("player_1".to_string());
    println!("Battle created with ID: {}", battle_id);

    // ... The rest of the test remains exactly the same ...
    println!("\nTurn 1: Using Solar Beam...");
    let action_request = SubmitActionRequest {
        battle_id,
        player_id: player_id.clone(),
        action: PlayerAction::UseMove { move_index: 1 }, // Solar Beam
    };
    let action_response = handler.submit_action(action_request).await.unwrap();
    assert!(action_response.success, "Action submission should succeed");

    // 3. Verify Turn 1 State (Charging)
    let events_request = GetBattleEventsRequest {
        battle_id,
        player_id: player_id.clone(),
        last_turns: Some(1),
    };
    let events_response = handler.get_battle_events(events_request).await.unwrap();
    let turn_1_events = &events_response.turn_logs[0].events;
    
    println!("Turn 1 Events:");
    turn_1_events.iter().for_each(|e| println!("  - {}", e));

    assert!(
        turn_1_events.iter().any(|e| e.contains("Venusaur was affected by Charging")),
        "Venusaur should have the Charging condition"
    );
    assert!(
        !turn_1_events.iter().any(|e| e.contains("took") && e.contains("damage")),
        "Solar Beam should not deal damage on the first turn"
    );
    
    let state_request = GetBattleStateRequest { battle_id, player_id: player_id.clone() };
    let state_response = handler.get_battle_state(state_request.clone()).await.unwrap();
    assert_eq!(state_response.turn_number, 2, "Should have advanced to turn 2");
    assert_eq!(state_response.game_state, pokemon_adventure::battle::state::GameState::WaitingForActions, "Should be waiting for actions for next turn");
    assert!(state_response.can_act, "Player should be able to act on turn 2, as the forced move is handled by the engine.");

    // 4. Turn 2: Player's action is submitted, but the engine should force Solar Beam
    println!("\nTurn 2: Submitting a placeholder action to trigger turn resolution...");
    let placeholder_action = SubmitActionRequest {
        battle_id,
        player_id: player_id.clone(),
        action: PlayerAction::UseMove { move_index: 0 }, // Engine will override this with Solar Beam
    };
    handler.submit_action(placeholder_action).await.unwrap();

    // 5. Verify Turn 2 State (Execution)
    let events_request_2 = GetBattleEventsRequest {
        battle_id,
        player_id: player_id.clone(),
        last_turns: Some(1),
    };
    let events_response_2 = handler.get_battle_events(events_request_2).await.unwrap();
    let turn_2_events = &events_response_2.turn_logs[0].events;

    println!("Turn 2 Events:");
    turn_2_events.iter().for_each(|e| println!("  - {}", e));

    assert!(
        turn_2_events.iter().any(|e| e.contains("Venusaur used SolarBeam")),
        "Solar Beam should be automatically used on the second turn"
    );
    assert!(
        turn_2_events.iter().any(|e| e.contains("took") && e.contains("damage")),
        "Solar Beam should deal damage on the second turn"
    );

    let final_state = handler.get_battle_state(state_request).await.unwrap();
    assert_eq!(final_state.turn_number, 3, "Should now be turn 3");
    assert!(final_state.can_act, "Player should be able to act again after Solar Beam executes");
    println!("\n✅ test_solar_beam_two_turn_flow PASSED");
}


#[tokio::test]
async fn test_fainting_and_npc_replacement_flow() {
    println!("\n--- Testing Fainting and NPC Replacement Flow ---");

    // 1. Setup: Create a battle
    let handler = create_test_handler().unwrap();
    let create_request = CreateMvpBattleRequest {
        player_name: "Test Trainer".to_string(),
        team_id: "charizard_team".to_string(), // A strong offensive team
        opponent_id: "gym_leader_easy".to_string(),
    };
    let create_response = handler.create_mvp_battle(create_request).await.unwrap();
    let battle_id = create_response.battle_id;
    let player_id = PlayerId("player_1".to_string());
    println!("Battle created with ID: {}", battle_id);

    // ... The rest of the test remains exactly the same ...
    let mut turn = 1;
    loop {
        println!("\nTurn {}: Attacking to cause a faint...", turn);
        let action_request = SubmitActionRequest {
            battle_id,
            player_id: player_id.clone(),
            action: PlayerAction::UseMove { move_index: 0 },
        };
        handler.submit_action(action_request).await.unwrap();

        let state_request = GetBattleStateRequest { battle_id, player_id: player_id.clone() };
        let state_response = handler.get_battle_state(state_request).await.unwrap();
        
        let opponent_hp = state_response.opponent_info.active_pokemon.as_ref().unwrap().current_hp;
        println!("Opponent HP: {}", opponent_hp);
        
        if state_response.game_state == pokemon_adventure::battle::state::GameState::WaitingForActions && opponent_hp == 0 {
             println!("Opponent fainted, checking for replacement...");
        }

        if opponent_hp == 0 {
             let events_request = GetBattleEventsRequest { battle_id, player_id: player_id.clone(), last_turns: Some(1) };
             let events_response = handler.get_battle_events(events_request).await.unwrap();
             let last_turn_events = &events_response.turn_logs.last().unwrap().events;
             
             assert!(
                 last_turn_events.iter().any(|e| e.contains("fainted!")),
                 "A fainted event should have occurred."
             );
             assert!(
                 last_turn_events.iter().any(|e| e.contains("Opponent recalled") && e.contains("sent out")),
                 "NPC should have automatically switched Pokémon after the faint."
             );

             let new_opponent_pokemon = state_response.opponent_info.active_pokemon.as_ref().unwrap();
             assert!(new_opponent_pokemon.current_hp > 0, "New NPC Pokémon should be healthy.");
             assert_eq!(state_response.game_state, pokemon_adventure::battle::state::GameState::WaitingForActions, "Game should be ready for the next turn after NPC replacement.");

             println!("\n✅ test_fainting_and_npc_replacement_flow PASSED");
             break;
        }

        turn += 1;
        if turn > 10 {
            panic!("Test failed: No faint occurred after 10 turns.");
        }
    }
}