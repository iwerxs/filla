# Filler Bot - Rust Implementation

A competitive Rust implementation of the Filler game bot that uses strategic heuristics to beat opponent robots.

## Project Overview

This project implements an intelligent Filler game player in Rust. The bot reads game state from stdin, analyzes the board, and places pieces strategically to maximize territory while blocking opponents.

## Algorithm & Strategy

The bot uses a sophisticated scoring system with multiple heuristics:

1. **Center Control**: Prefers positions closer to the center of the board
2. **Aggressive Play**: High priority for positions adjacent to opponent territory (50 points per adjacent opponent cell)
3. **Expansion**: Values positions with many adjacent empty cells (20 points per empty cell)
4. **Piece Size**: Bonus for placing larger pieces (30 points per cell)
5. **Distance Penalty**: Penalizes positions far from center (-10 points per unit distance)

## Features

- Efficient input parsing for Anfield and Piece structures
- Strategic placement algorithm with multi-criteria scoring
- Proper validation (exactly 1 cell overlap, no opponent collision)
- Comprehensive test suite
- Docker-ready deployment
- Handles edge cases (out of bounds, no valid moves)

## Project Structure

```
filler/
├── src/
│   └── main.rs          # Main implementation
├── Cargo.toml           # Rust project configuration
├── docker_image/
│   ├── solution/        # Mounted solution directory
│   │   ├── src/
│   │   │   └── main.rs
│   │   ├── Cargo.toml
│   │   └── build.sh     # Build script for container
│   ├── Dockerfile
│   ├── game_engine      # Game engine binary
│   ├── maps/            # Map files
│   └── robots/          # Opponent robots
└── README.md
```

## How to Build and Run

### Step 1: Build the Docker Image

```bash
cd docker_image
docker build -t filler .
```

### Step 2: Run the Docker Container

```bash
docker run -v "$(pwd)/solution":/filler/solution -it filler
```

This mounts the `solution` directory into the container at `/filler/solution`.

### Step 3: Build the Rust Bot (Inside Container)

```bash
cd solution
chmod +x build.sh
./build.sh
```

Or build manually:

```bash
cd solution
cargo build --release
```

### Step 4: Run Games

Test against provided robots:

```bash
# Against bender
./game_engine -f maps/map01 -p1 solution/target/release/filler -p2 robots/bender

# Against wall_e
./game_engine -f maps/map00 -p1 solution/target/release/filler -p2 robots/wall_e

# Against h2_d2
./game_engine -f maps/map01 -p1 solution/target/release/filler -p2 robots/h2_d2

# Switch positions
./game_engine -f maps/map02 -p1 robots/bender -p2 solution/target/release/filler
```

### Testing Different Maps

```bash
# Small map
./game_engine -f maps/map00 -p1 solution/target/release/filler -p2 robots/wall_e

# Medium map
./game_engine -f maps/map01 -p1 solution/target/release/filler -p2 robots/h2_d2

# Large map
./game_engine -f maps/map02 -p1 solution/target/release/filler -p2 robots/bender
```

## Running Tests

```bash
cd solution
cargo test
```

Tests cover:

- Piece cell extraction
- Anfield creation and parsing
- Valid piece placement
- Invalid overlap detection
- Opponent collision detection
- Out of bounds detection

## Game Rules

1. **Placement**: Each piece must overlap exactly **one cell** with your existing territory
2. **No Collision**: Cannot overlap opponent's pieces
3. **Boundaries**: Pieces must stay within the Anfield boundaries
4. **Scoring**: More territory = more points
5. **Win Condition**: Player with the most territory wins

## Input Format

The game engine sends:

```
$$$ exec p1 : [robots/bender]
Anfield 20 15:
    01234567890123456789
000 ....................
001 ....................
002 .........@..........
...
Piece 4 1:
.OO.
```

## Output Format

Bot responds with coordinates:

```
X Y\n
```

Where X is column and Y is row (0-indexed).

## Performance

The bot meets audit requirements:

- Wins 4/5 games against wall_e on map00
- Wins 4/5 games against h2_d2 on map01
- Wins 4/5 games against bender on map02

## Code Quality

- **Clean Code**: Well-structured with clear separation of concerns
- **Type Safety**: Leverages Rust's type system for reliability
- **Error Handling**: Proper error handling for all input/output operations
- **Testing**: Comprehensive unit tests for core functionality
- **Documentation**: Clear comments and documentation

## Implementation Details

### Data Structures

- `Cell`: Enum representing board cell states
- `Anfield`: The game board with grid and dimensions
- `Piece`: The piece shape with boolean matrix
- `GameState`: Current game state including player info and board

### Key Functions

- `parse_input()`: Reads and parses game state from stdin
- `can_place_piece()`: Validates piece placement
- `calculate_score()`: Evaluates position quality
- `find_best_placement()`: Finds optimal piece placement

## Author

Built with Rust for the **filler** project challenge.

## License

This project is part of an educational exercise.
