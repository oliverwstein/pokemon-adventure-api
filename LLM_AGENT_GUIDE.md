# Pokemon Adventure API - LLM Agent Integration Guide

## Overview

This guide explains how to interact with the Pokemon Adventure API as an LLM agent. The API provides a complete Pokemon battle system with Generation 1 mechanics, accessible through standard HTTP requests.

**Base URL**: `https://your-api-gateway-url/prod` (replace with your actual API Gateway URL)

## Quick Start Flow

### 1. Explore Available Options
First, discover what teams and opponents are available:

```bash
# Get available Pokemon teams
GET /available_teams

# Get NPC opponents
GET /npc_opponents
```

### 2. Create a Battle
Choose a team and opponent, then create a battle:

```bash
POST /battles
{
  "player_name": "Agent_Claude",
  "team_id": "venusaur_team",
  "opponent_id": "brock"
}
```

### 3. Battle Loop
Repeat until battle ends:

```bash
# Check what actions you can take
GET /battles/{battle_id}/valid_actions

# Submit your action
POST /battles/{battle_id}/action
{
  "action_type": "UseMove",
  "move_name": "Vine Whip"
}

# Check battle state after turn
GET /battles/{battle_id}/state
```

## Detailed API Reference

### Available Teams

**Endpoint**: `GET /available_teams`

**Response**:
```json
{
  "teams": [
    {
      "id": "venusaur_team",
      "name": "Grass Masters",
      "description": "Venusaur-led team with strong Grass/Poison types",
      "pokemon": [
        {
          "species": "Venusaur",
          "level": 60,
          "moves": ["Vine Whip", "Sleep Powder", "Petal Dance", "Earthquake"]
        }
        // ... 5 more Pokemon
      ]
    }
    // ... other teams
  ]
}
```

**Available Teams**:
- `venusaur_team`: Grass/Poison specialists with status moves
- `blastoise_team`: Water types with defensive strategies
- `charizard_team`: Fire/Flying with aggressive tactics

### NPC Opponents

**Endpoint**: `GET /npc_opponents`

**Response**:
```json
{
  "opponents": [
    {
      "id": "brock",
      "name": "Brock",
      "difficulty": "Easy",
      "description": "Rock-type Gym Leader with defensive strategies"
    }
    // ... other opponents
  ]
}
```

**Available Opponents**:
- `brock` (Easy): Rock-type specialist, defensive play
- `misty` (Medium): Water-type master, balanced tactics  
- `lt_surge` (Hard): Electric-type powerhouse, aggressive

### Create Battle

**Endpoint**: `POST /battles`

**Request**:
```json
{
  "player_name": "YourAgentName",
  "team_id": "venusaur_team",
  "opponent_id": "brock"
}
```

**Response**:
```json
{
  "battle_id": "battle_abc123",
  "message": "Battle created successfully",
  "initial_state": {
    // Complete battle state...
  }
}
```

### Get Battle State

**Endpoint**: `GET /battles/{battle_id}/state`

**Response**:
```json
{
  "battle_id": "battle_abc123",
  "player": {
    "name": "YourAgentName",
    "active_pokemon": {
      "species": "Venusaur",
      "level": 60,
      "current_hp": 180,
      "max_hp": 180,
      "status": null,
      "stat_stages": {
        "attack": 0,
        "defense": 0,
        "special_attack": 0,
        "special_defense": 0,
        "speed": 0
      }
    },
    "team_size": 6,
    "can_act": true
  },
  "opponent": {
    // Similar structure for NPC
  },
  "turn_number": 1,
  "battle_status": "InProgress",
  "recent_events": [
    "Battle started!",
    "Venusaur was sent out!",
    "Onix was sent out!"
  ]
}
```

### Get Valid Actions

**Endpoint**: `GET /battles/{battle_id}/valid_actions`

**Response**:
```json
{
  "battle_id": "battle_abc123",
  "can_act": true,
  "available_moves": [
    {
      "name": "Vine Whip",
      "type": "Grass",
      "category": "Physical",
      "power": 45,
      "accuracy": 100,
      "pp_remaining": 25,
      "description": "Strikes with vines"
    },
    {
      "name": "Sleep Powder",
      "type": "Grass", 
      "category": "Status",
      "power": null,
      "accuracy": 75,
      "pp_remaining": 15,
      "description": "Induces sleep"
    }
    // ... other moves
  ],
  "can_switch": true,
  "available_switches": [
    {
      "species": "Vileplume",
      "level": 60,
      "current_hp": 160,
      "max_hp": 160,
      "status": null
    }
    // ... other team members
  ]
}
```

### Submit Action

**Endpoint**: `POST /battles/{battle_id}/action`

**Move Action**:
```json
{
  "action_type": "UseMove",
  "move_name": "Vine Whip"
}
```

**Switch Action**:
```json
{
  "action_type": "Switch",
  "pokemon_species": "Vileplume"
}
```

**Forfeit Action**:
```json
{
  "action_type": "Forfeit"
}
```

**Response**:
```json
{
  "battle_id": "battle_abc123",
  "action_result": "success",
  "updated_state": {
    // Complete battle state after processing turn
  },
  "events": [
    "Venusaur used Vine Whip!",
    "It's super effective!",
    "Onix took 89 damage!",
    "Onix used Rock Throw!",
    "Venusaur took 45 damage!"
  ]
}
```

## Battle Mechanics for Agents

### Type Effectiveness
Understanding type matchups is crucial for optimal play:

**Super Effective (2x damage)**:
- Water → Rock, Fire, Ground
- Grass → Water, Rock, Ground  
- Fire → Grass, Ice, Bug, Steel
- Electric → Water, Flying

**Not Very Effective (0.5x damage)**:
- Water → Water, Grass, Dragon
- Fire → Fire, Water, Rock, Dragon
- Grass → Fire, Grass, Poison, Flying, Bug, Dragon, Steel

**No Effect (0x damage)**:
- Normal → Ghost
- Electric → Ground
- Ground → Flying

### Status Conditions
- **Sleep**: Prevents move usage for 1-7 turns
- **Poison**: Deals 1/8 max HP damage each turn
- **Burn**: Halves physical attack + 1/16 HP damage per turn  
- **Paralysis**: 25% chance to skip turn + 50% speed reduction
- **Freeze**: Cannot move until thawed (rare)

### Strategy Tips

#### Move Selection
1. **Check type effectiveness** before attacking
2. **Use status moves** early (Sleep Powder, Thunder Wave)
3. **Consider PP management** for long battles
4. **Watch for critical hits** (high-speed Pokemon more likely)

#### Switching Strategy
1. **Switch when disadvantaged** by type matchups
2. **Preserve strong Pokemon** for late game
3. **Use switches to heal** status conditions
4. **Scout opponent moves** with expendable Pokemon

#### Advanced Tactics
1. **Stat stage moves** provide lasting advantages (Swords Dance, Amnesia)
2. **Multi-hit moves** can break through Substitute
3. **Priority moves** (Quick Attack) ignore speed
4. **Two-turn moves** (Solar Beam, Dig) telegraph but hit hard

## Error Handling

### Common Errors

**400 Bad Request**:
```json
{
  "error": "Invalid action: Move 'Flamethrower' not available"
}
```
- Action not in `valid_actions` list
- Pokemon fainted or cannot act
- Invalid move/Pokemon name

**404 Not Found**:
```json
{
  "error": "Battle battle_xyz not found"
}
```
- Battle ID doesn't exist
- Battle may have expired

**500 Internal Server Error**:
```json
{
  "error": "Database error: Unable to save battle state"
}
```
- Temporary service issue
- Retry after brief delay

### Error Recovery
1. **Always check `can_act`** before submitting actions
2. **Validate moves** against `valid_actions` response
3. **Handle timeouts** gracefully with retries
4. **Check battle status** - may have ended

## Sample Battle Flow

Here's a complete example of an agent playing a battle:

```python
import requests
import json

BASE_URL = "https://your-api-gateway-url/prod"

# 1. Create battle
battle_response = requests.post(f"{BASE_URL}/battles", json={
    "player_name": "AI_Agent",
    "team_id": "venusaur_team", 
    "opponent_id": "brock"
})
battle_id = battle_response.json()["battle_id"]

# 2. Battle loop
while True:
    # Check if we can act
    state = requests.get(f"{BASE_URL}/battles/{battle_id}/state").json()
    
    if not state["player"]["can_act"]:
        print("Waiting for opponent...")
        continue
        
    if state["battle_status"] != "InProgress":
        print(f"Battle ended: {state['battle_status']}")
        break
    
    # Get available actions
    actions = requests.get(f"{BASE_URL}/battles/{battle_id}/valid_actions").json()
    
    # Simple AI: Use first super effective move, or first available move
    move_to_use = actions["available_moves"][0]["name"]  # Default
    
    for move in actions["available_moves"]:
        if "super effective" in move.get("description", "").lower():
            move_to_use = move["name"]
            break
    
    # Submit action
    action_response = requests.post(f"{BASE_URL}/battles/{battle_id}/action", json={
        "action_type": "UseMove",
        "move_name": move_to_use
    })
    
    # Print battle events
    events = action_response.json()["events"]
    for event in events:
        print(event)
```

## Best Practices for LLM Agents

### Performance
1. **Batch requests** when possible (get state + valid actions together)
2. **Cache team/opponent data** to avoid repeated calls
3. **Use appropriate timeouts** (30s max per request)
4. **Handle rate limits** gracefully

### Strategy
1. **Analyze opponent patterns** from battle events
2. **Plan team composition** based on opponent type
3. **Manage resources** (HP, PP) across the team
4. **Adapt strategy** based on battle state

### Reliability
1. **Always validate responses** before using data
2. **Implement retry logic** for transient failures
3. **Handle partial failures** gracefully
4. **Log important decisions** for debugging

### Battle Analysis
Monitor these key metrics:
- **Damage efficiency**: Damage dealt vs. damage taken
- **Type advantage usage**: How often you get super effective hits
- **Status move success**: Rate of status condition application
- **Switch timing**: When to preserve vs. sacrifice Pokemon

This API provides a complete Pokemon battle experience with authentic Generation 1 mechanics. The deterministic engine ensures fair, predictable gameplay perfect for AI agent development and testing.