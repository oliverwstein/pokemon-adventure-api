# Pokemon Adventure API

REST API for the Pokemon Adventure battle system, designed for AWS Lambda deployment with DynamoDB storage.

## Architecture

This API crate provides:
- **Stateless Lambda handlers** for Pokemon battle management
- **DynamoDB integration** for persistent battle state storage
- **RESTful endpoints** for battle creation, action submission, and state retrieval
- **Engine integration** using the compile-time optimized pokemon-adventure crate

## API Endpoints

### Create Battle
```
POST /battles
```
Creates a new Pokemon battle between two players.

**Request Body:**
```json
{
  "player1_id": "player1",
  "player2_id": "player2", 
  "player1_team": [
    {
      "species": "Pikachu",
      "level": 25,
      "moves": ["QuickAttack", "Thunderclap", "TailWhip", "Agility"],
      "nickname": "Sparky"
    }
  ],
  "player2_team": [...]
}
```

### Submit Action
```
POST /battles/{battle_id}/actions
```
Submit a player action (move, switch, forfeit).

**Request Body:**
```json
{
  "player_id": "player1",
  "action": {
    "UseMove": { "move_index": 0 }
  }
}
```

### Get Battle State
```
GET /battles/{battle_id}?player_id={player_id}
```
Retrieve current battle state and status.

## Deployment

This crate is designed for deployment using `cargo lambda`:

```bash
cargo lambda build --release
cargo lambda deploy
```

## Environment Variables

- `DYNAMODB_TABLE_NAME`: DynamoDB table name for battle storage (default: "pokemon-battles")

## Database Schema

### DynamoDB Table: pokemon-battles

**Primary Key:** `battle_id` (String)

**Attributes:**
- `battle_id`: UUID string
- `player1_id`: String
- `player2_id`: String  
- `battle_state`: JSON string (serialized BattleState)
- `created_at`: Unix timestamp
- `last_updated`: Unix timestamp

## Performance Benefits

- **Zero cold start file loading** - All Pokemon/move data compiled at build time
- **Optimized serialization** - Efficient BattleState persistence
- **Stateless design** - Perfect for Lambda scaling and reliability