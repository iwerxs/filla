// tactics.rs

use crate::pitch::{Pitch, Position};
use crate::game_analysis::PotentialMove;
use crate::grid_matrix::Grid;
use crate::object::Object;

/// Stores adjacent cells around a position (up, down, left, right)
pub struct AdjacentCells {
    pub up: Option<char>,
    pub down: Option<char>,
    pub left: Option<char>,
    pub right: Option<char>,
}

/// Calculates the average (center) position of a robot's objects
/// Used for strategic targeting and positioning
pub fn calculate_average_position(
    pitch: &Pitch,
    our_symbols: (char, char),
    find_opponent: bool, // true = find opponent center, false = find our center
) -> Position {
    let mut total_row = 0;
    let mut total_col = 0;
    let mut cell_count = 0;

    // Scan the whole pitch
    for row in 0..pitch.dimensions.height {
        for col in 0..pitch.dimensions.width {
            let cell = pitch.cells[row][col];

            let is_target_cell = if find_opponent {
                is_opponent_cell(Some(cell), our_symbols)
            } else {
                is_our_cell(Some(cell), our_symbols)
            };

            if is_target_cell {
                total_row += row;
                total_col += col;
                cell_count += 1;
            }
        }
    }

    // Calculate average position
    if cell_count > 0 {
        Position {
            row: total_row / cell_count,
            col: total_col / cell_count,
        }
    } else {
        // No cells found - default to center of pitch
        Position {
            row: pitch.dimensions.height / 2,
            col: pitch.dimensions.width / 2,
        }
    }
}

/// Gets the cells directly adjacent to a position (up, down, left, right)
pub fn get_adjacent_cells(pitch: &Pitch, position: &Position) -> AdjacentCells {
    let mut adjacent = AdjacentCells {
        up: None,
        down: None,
        left: None,
        right: None,
    };

    // Check up
    if position.row > 0 {
        adjacent.up = Some(pitch.cells[position.row - 1][position.col]);
    }

    // Check down
    if position.row + 1 < pitch.height() {
        adjacent.down = Some(pitch.cells[position.row + 1][position.col]);
    }

    // Check left
    if position.col > 0 {
        adjacent.left = Some(pitch.cells[position.row][position.col - 1]);
    }

    // Check right
    if position.col + 1 < pitch.width() {
        adjacent.right = Some(pitch.cells[position.row][position.col + 1]);
    }

    adjacent
}

/// Main evaluation function - picks the best move from all valid placements
/// Applies multiple strategic heuristics and weights them
pub fn evaluate_all_placements(
    pitch: &Pitch,
    mut valid_moves: Vec<PotentialMove>,
    opponent_center: Position,
    current_turn: usize,
    our_symbols: (char, char),
    previous_objects: &Vec<Object>,
) -> PotentialMove {
    // Check if we're already touching the opponent
    let touching_opponent = are_we_touching_opponent(pitch, our_symbols);

    // STRATEGY 1: Chase opponent (early-mid game)
    // Move closer to opponent's center of mass
    if !touching_opponent {
        score_by_distance_to_opponent(pitch, &mut valid_moves, opponent_center, current_turn);
    }

    // STRATEGY 2: Trap/Enclose opponent cells
    // Prioritize moves that surround opponent robot
    let found_trapping_opportunities =
        score_by_enclosure_potential(pitch, &mut valid_moves, our_symbols);

    // STRATEGY 3: Fill gaps efficiently (late game)
    // Prefer moves that perfectly fit into spaces with no waste
    if !found_trapping_opportunities {
        score_by_perfect_fit(
            pitch,
            &mut valid_moves,
            current_turn,
            previous_objects,
            our_symbols,
        );
    }

    // Return the highest scoring move
    valid_moves
        .into_iter()
        .max_by_key(|placement| placement.score)
        .unwrap()
}

/// STRATEGY 1: Score moves by proximity to opponent
/// Closer moves get higher scores (helps us chase and pressure opponent)
/// Weight decreases as game progresses (less important late game)
fn score_by_distance_to_opponent(
    pitch: &Pitch,
    placements: &mut Vec<PotentialMove>,
    opponent_center: Position,
    turn_number: usize,
) {
    // List of (distance, index) for sorting
    let mut distances: Vec<(f32, usize)> = Vec::new();

    for (index, placement) in placements.iter().enumerate() {
        // Calculate objects center position
        let object_center = calculate_object_center(pitch, &placement.position, &placement.object);

        // Manhattan distance to opponent center
        let distance = calculate_distance(object_center, opponent_center);
        distances.push((distance, index));
    }

    // Sort by distance (closest first)
    distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Calculate score bonus (decreases each turn)
    // Early game: high bonus (~36 points)
    // Late game: low bonus (~4 points)
    let max_bonus = ((36.0 * (2.0_f32).powf(-0.15 * turn_number as f32)) as i32).max(4);

    // Create score tiers that decrease by half each time
    let mut score_tiers = Vec::new();
    let mut current_bonus = max_bonus;
    while current_bonus > 1 {
        score_tiers.push(current_bonus);
        current_bonus /= 2;
    }

    // Award bonuses to closest moves
    for (tier_index, &(_distance, move_index)) in
        distances.iter().take(score_tiers.len()).enumerate()
    {
        placements[move_index].score += score_tiers[tier_index];
    }
}

/// STRATEGY 2: Score moves that trap/enclose opponent cells
/// Returns true if any enclosure opportunities were found
fn score_by_enclosure_potential(
    pitch: &Pitch,
    placements: &mut Vec<PotentialMove>,
    our_symbols: (char, char),
) -> bool {
    let enclosure_bonus = 12;
    let mut found_opportunities = false;

    // Find positions that would enclose opponent cells
    let enclosure_positions = find_enclosure_positions(pitch, our_symbols);

    for placement in placements.iter_mut() {
        // Check each cell this placement would fill
        for object_row in 0..placement.object.trimmed_dimensions.height {
            for object_col in 0..placement.object.trimmed_dimensions.width {
                let object_cell = placement.object.trimmed_cells[object_row][object_col];

                // Only score filled cells
                if object_cell != 'O' {
                    continue;
                }

                let pitch_row = placement.position.row + object_row;
                let pitch_col = placement.position.col + object_col;

                // Check if this position encloses an opponent cell
                let mut best_enclosure_value = 3; // Lower = better enclosure

                for &(ref enclosure_pos, enclosure_distance) in &enclosure_positions {
                    if enclosure_pos.row == pitch_row && enclosure_pos.col == pitch_col {
                        if enclosure_distance < best_enclosure_value {
                            best_enclosure_value = enclosure_distance;
                            if enclosure_distance == 1 {
                                found_opportunities = true;
                                break;
                            }
                        }
                    }
                }

                // Award bonus inversely proportional to distance
                // Distance 1 (adjacent) = full bonus (12 points)
                // Distance 2 = half bonus (6 points)
                // Distance 3 = third bonus (4 points)
                placement.score += enclosure_bonus / best_enclosure_value as i32;
            }
        }
    }

    found_opportunities
}

/// STRATEGY 3: Score moves that perfectly fill gaps with no wasted space
/// More important late game when pitch is filling up
fn score_by_perfect_fit(
    pitch: &Pitch,
    placements: &mut Vec<PotentialMove>,
    turn_number: usize,
    _previous_objects: &Vec<Object>,
    _our_symbols: (char, char),
) {
    // Calculate bonus (grows exponentially with turn number)
    // Early game: ~1 point
    // Late game: ~50 points (capped)
    let perfect_fit_bonus = ((1.07_f32).powi(turn_number as i32) as i32).min(50);

    for placement in placements.iter_mut() {
        let mut is_perfect_fit = true;

        // Check if object fills space with no gaps
        let top_left = placement.position.clone();
        let bottom_right = Position {
            row: placement.position.row + placement.object.original_dimensions.height,
            col: placement.position.col + placement.object.original_dimensions.width,
        };

        // Scan the bounding box
        'outer: for row in top_left.row..bottom_right.row {
            for col in top_left.col..bottom_right.col {
                // Skip positions outside of the Pitch
                if row >= pitch.height() || col >= pitch.width() {
                    continue;
                }

                let pitch_cell = pitch.cells[row][col];
                let object_cell = placement.object.cells()[row - top_left.row][col - top_left.col];

                // If both Pitch and Objects are empty here = wasted space
                if pitch_cell == '.' && object_cell == '.' {
                    is_perfect_fit = false;
                    break 'outer;
                }
            }
        }

        if is_perfect_fit {
            placement.score += perfect_fit_bonus;
        }
    }
}

/// Checks if our Robot is already adjacent to opponent robot
fn are_we_touching_opponent(pitch: &Pitch, our_symbols: (char, char)) -> bool {
    for row in 0..pitch.height() {
        for col in 0..pitch.width() {
            let cell = pitch.cells[row][col];

            // Found an opponent cell
            if is_opponent_cell(Some(cell), our_symbols) {
                let adjacent = get_adjacent_cells(pitch, &Position { row, col });

                // Check if any adjacent cell is ours
                if (adjacent.up.is_some() && is_our_cell(adjacent.up, our_symbols))
                    || (adjacent.down.is_some() && is_our_cell(adjacent.down, our_symbols))
                    || (adjacent.left.is_some() && is_our_cell(adjacent.left, our_symbols))
                    || (adjacent.right.is_some() && is_our_cell(adjacent.right, our_symbols))
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Finds positions that would trap opponent cells
/// Returns: Vec<(Position, distance_from_opponent)>
/// Distance 1 = directly adjacent, 2 = one cell away, 3 = two cells away
fn find_enclosure_positions(pitch: &Pitch, our_symbols: (char, char)) -> Vec<(Position, usize)> {
    let mut positions = Vec::new();

    for row in 0..pitch.height() {
        for col in 0..pitch.width() {
            let cell = pitch.cells[row][col];

            // Found opponent cell - check positions around it
            if is_opponent_cell(Some(cell), our_symbols) {
                // Check 1-3 cells away in each direction
                for distance in 1..=3 {
                    // Up
                    if row >= distance {
                        let target = pitch.cells[row - distance][col];
                        if target == '.' {
                            positions.push((
                                Position {
                                    row: row - distance,
                                    col,
                                },
                                distance,
                            ));
                        }
                    }

                    // Down
                    if row + distance < pitch.height() {
                        let target = pitch.cells[row + distance][col];
                        if target == '.' {
                            positions.push((
                                Position {
                                    row: row + distance,
                                    col,
                                },
                                distance,
                            ));
                        }
                    }

                    // Left
                    if col >= distance {
                        let target = pitch.cells[row][col - distance];
                        if target == '.' {
                            positions.push((
                                Position {
                                    row,
                                    col: col - distance,
                                },
                                distance,
                            ));
                        }
                    }

                    // Right
                    if col + distance < pitch.width() {
                        let target = pitch.cells[row][col + distance];
                        if target == '.' {
                            positions.push((
                                Position {
                                    row,
                                    col: col + distance,
                                },
                                distance,
                            ));
                        }
                    }
                }
            }
        }
    }

    positions
}

/// Utility: Check if a cell belongs to opponent
fn is_opponent_cell(cell: Option<char>, our_symbols: (char, char)) -> bool {
    if let Some(c) = cell {
        c != '.' && c != our_symbols.0 && c != our_symbols.1
    } else {
        false
    }
}

/// Utility: Check if a cell belongs to us
fn is_our_cell(cell: Option<char>, our_symbols: (char, char)) -> bool {
    if let Some(c) = cell {
        c == our_symbols.0 || c == our_symbols.1
    } else {
        false
    }
}

/// Utility: Calculate center position of a placed object (robot)
fn calculate_object_center(pitch: &Pitch, position: &Position, object: &Object) -> Position {
    let mut center_row = position.row + (object.trimmed_dimensions.height + object.trim_offset.0) / 2;
    let mut center_col = position.col + (object.trimmed_dimensions.width + object.trim_offset.1) / 2;

    // Clamp to pitch boundaries
    if center_row >= pitch.height() {
        center_row = pitch.height() - 1;
    }
    if center_col >= pitch.width() {
        center_col = pitch.width() - 1;
    }

    Position {
        row: center_row,
        col: center_col,
    }
}

/// Utility: Calculate Euclidean distance between two positions
fn calculate_distance(pos1: Position, pos2: Position) -> f32 {
    let row_diff = (pos1.row as i32 - pos2.row as i32).pow(2);
    let col_diff = (pos1.col as i32 - pos2.col as i32).pow(2);
    ((row_diff + col_diff) as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid_matrix::Dimensions;

    fn create_test_pitch() -> Pitch {
        Pitch {
            dimensions: Dimensions {
                width: 4,
                height: 4,
            },
            cells: vec![
                vec!['.', 'a', '.', '.'],
                vec!['.', '.', '.', '.'],
                vec!['.', '.', 's', '.'],
                vec!['.', '.', '.', '.'],
            ],
        }
    }

    #[test]
    fn test_are_we_touching_opponent() {
        let pitch = create_test_pitch();
        let our_symbols = ('a', '@');

        // Not touching initially
        let touching = are_we_touching_opponent(&pitch, our_symbols);
        assert_eq!(touching, false);

        // Create pitch where we're adjacent
        let mut adjacent_pitch = create_test_pitch();
        adjacent_pitch.cells[1][2] = 'a'; // Place our robot next to 's' at (2,2)

        let touching = are_we_touching_opponent(&adjacent_pitch, our_symbols);
        assert_eq!(touching, true);
    }

    #[test]
    fn test_is_opponent_cell() {
        let our_symbols = ('a', '@');

        assert_eq!(is_opponent_cell(Some('s'), our_symbols), true);
        assert_eq!(is_opponent_cell(Some('$'), our_symbols), true);
        assert_eq!(is_opponent_cell(Some('a'), our_symbols), false);
        assert_eq!(is_opponent_cell(Some('.'), our_symbols), false);
    }

    #[test]
    fn test_is_our_cell() {
        let our_symbols = ('a', '@');

        assert_eq!(is_our_cell(Some('a'), our_symbols), true);
        assert_eq!(is_our_cell(Some('@'), our_symbols), true);
        assert_eq!(is_our_cell(Some('s'), our_symbols), false);
        assert_eq!(is_our_cell(Some('.'), our_symbols), false);
    }
}
