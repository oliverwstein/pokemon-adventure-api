# Pokemon Adventure API - MVP Test Guide

## 🎯 **MVP API Endpoints**

The following endpoints are now implemented and ready for testing:

### 1. **GET /available_teams** - List Prefab Teams
```bash
curl -X GET "https://your-api-gateway-url/available_teams"
```

**Response:**
```json
{
  "teams": [
    {
      "id": "venusaur_team",
      "name": "Venusaur Team", 
      "description": "Elite team featuring Venusaur with diverse type coverage",
      "pokemon_count": 6,
      "average_level": 60
    },
    {
      "id": "blastoise_team",
      "name": "Blastoise Team",
      "description": "Balanced team featuring Blastoise with excellent type diversity", 
      "pokemon_count": 6,
      "average_level": 60
    },
    {
      "id": "charizard_team", 
      "name": "Charizard Team",
      "description": "Aggressive team featuring Charizard with high offensive potential",
      "pokemon_count": 6,
      "average_level": 60
    }
  ]
}
```

### 2. **GET /npc_opponents** - List NPC Opponents
```bash
curl -X GET "https://your-api-gateway-url/npc_opponents"
```

**Response:**
```json
{
  "opponents": [
    {
      "id": "gym_leader_easy",
      "name": "Gym Leader Brock",
      "description": "Rock-type specialist with defensive strategies",
      "difficulty": "easy"
    },
    {
      "id": "gym_leader_medium", 
      "name": "Gym Leader Misty",
      "description": "Water-type master with balanced offense and control",
      "difficulty": "medium"
    },
    {
      "id": "gym_leader_hard",
      "name": "Gym Leader Lt. Surge", 
      "description": "Electric-type powerhouse with aggressive tactics",
      "difficulty": "hard"
    }
  ]
}
```

### 3. **POST /battles** - Create Battle
```bash
curl -X POST "https://your-api-gateway-url/battles" \
  -H "Content-Type: application/json" \
  -d '{
    "player_name": "Ash Ketchum",
    "team_id": "venusaur_team",
    "opponent_id": "gym_leader_easy"
  }'
```

**Response:**
```json
{
  "battle_id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "Battle created successfully",
  "battle_state": {
    "battle_id": "123e4567-e89b-12d3-a456-426614174000",
    "game_state": "WaitingForActions",
    "turn_number": 1,
    "can_act": true,
    "player_team": {
      "active_pokemon": {
        "name": "Venusaur",
        "species": "Venusaur",
        "level": 60,
        "current_hp": 190,
        "max_hp": 190,
        "moves": [
          {"move_": "SleepPowder", "pp": 15, "max_pp": 15},
          {"move_": "SolarBeam", "pp": 10, "max_pp": 10},
          {"move_": "PetalDance", "pp": 10, "max_pp": 10},
          {"move_": "Earthquake", "pp": 10, "max_pp": 10}
        ]
      },
      "team_pokemon": [...]
    },
    "opponent_info": {
      "player_name": "NPC Trainer (easy)",
      "active_pokemon": {...},
      "remaining_pokemon_count": 6
    }
  }
}
```

### 4. **GET /battles/{battle_id}/state** - Get Battle State
```bash
curl -X GET "https://your-api-gateway-url/battles/{battle_id}/state?player_id=player_1"
```

### 5. **GET /battles/{battle_id}/valid_actions** - Get Valid Actions
```bash  
curl -X GET "https://your-api-gateway-url/battles/{battle_id}/valid_actions?player_id=player_1"
```

**Response:**
```json
{
  "battle_id": "123e4567-e89b-12d3-a456-426614174000",
  "valid_actions": [
    {"UseMove": {"move_index": 0}},
    {"UseMove": {"move_index": 1}}, 
    {"UseMove": {"move_index": 2}},
    {"UseMove": {"move_index": 3}},
    {"SwitchPokemon": {"team_index": 1}},
    {"SwitchPokemon": {"team_index": 2}},
    {"SwitchPokemon": {"team_index": 3}},
    {"SwitchPokemon": {"team_index": 4}},
    {"SwitchPokemon": {"team_index": 5}},
    {"Forfeit": null}
  ]
}
```

### 6. **POST /battles/{battle_id}/action** - Submit Action
```bash
curl -X POST "https://your-api-gateway-url/battles/{battle_id}/action" \
  -H "Content-Type: application/json" \
  -d '{
    "player_id": "player_1",
    "action": {"UseMove": {"move_index": 0}}
  }'
```

**Response:**
```json
{
  "success": true,
  "message": "Action processed successfully", 
  "battle_updated": true
}
```

## 🎮 **Complete Battle Flow Example**

1. **Choose Team**: `GET /available_teams` → Pick `venusaur_team`
2. **Choose Opponent**: `GET /npc_opponents` → Pick `gym_leader_easy`
3. **Create Battle**: `POST /battles` with team and opponent
4. **Get Actions**: `GET /battles/{id}/valid_actions` 
5. **Submit Move**: `POST /battles/{id}/action` with `UseMove`
6. **Check State**: `GET /battles/{id}/state` to see results
7. **Repeat steps 4-6** until battle ends

## 🏗️ **Architecture Implemented**

✅ **Clean Architecture**: Request → Router → DB → Engine → DB → Response  
✅ **Stateless Functions**: Pure game logic with no I/O dependencies  
✅ **Game Tick Loop**: AI processes multiple turns per action submission  
✅ **Player Authorization**: Battle state filtered by requesting player  
✅ **Robust Error Handling**: Typed errors with proper HTTP status codes  
✅ **Prefab Team System**: Predefined teams with balanced Pokemon  
✅ **NPC Opponents**: AI-controlled opponents with different difficulties  

## 🚀 **Ready for LLM Agent Testing**

This API is designed for LLM agents like Claude Code to:
- List available teams and opponents
- Create battles programmatically  
- Get valid actions at each turn
- Submit moves and see battle progression
- Complete full Pokemon battles autonomously

The API handles all the complex battle logic internally, so agents just need to make HTTP requests and parse JSON responses.