// game_analysis.rs

pub use crate::pitch::*;
pub use crate::object::*;
pub use crate::contestant::*;
pub use crate::tactics::*;

/// Main game state tracker
/// Manages the current game, contestant states, and placement history
#[derive(Debug, Clone)]
pub struct GameAnalysis {
    pub our_contestant: Contestant,
    pub opponent: Contestant,
    pub pitch: Pitch,
    pub placement_history: Vec<Object>,
    pub turn_number: usize,
}

/// Represents a potential object placement with its calculated score
#[derive(Debug, Clone)]
pub struct PotentialMove {
    pub position: Position,
    pub score: i32,
    pub object: Object,
}

impl GameAnalysis {
    /// Creates a new game state
    pub fn new(our_contestant: Contestant, opponent: Contestant, pitch: Pitch) -> Self {
        Self {
            our_contestant,
            opponent,
            pitch,
            placement_history: Vec::new(),
            turn_number: 0,
        }
    }

    /// Calculates the best move for the given object
    /// Returns (x, y) coordinates or (0, 0) if no valid placement exists
    pub fn calculate_best_move(&mut self, object: Object) -> (i32, i32) {
        self.turn_number += 1;

        // Quick check: object too big for pitch
        if object.trimmed_dimensions.height > self.pitch.dimensions.height
            || object.trimmed_dimensions.width > self.pitch.dimensions.width
        {
            return (0, 0);
        }

        // Calculate opponent's average position for strategic targeting
        let opponent_center = calculate_average_position(
            &self.pitch,
            self.our_contestant.symbols,
            true, // looking for opponent
        );

        // Find all valid placements for this object
        let mut valid_placements: Vec<PotentialMove> = Vec::new();

        // Try every possible position on the pitch
        // Start from object offset to avoid checking positions where object would be off-pitch
        let max_row = self.pitch.dimensions.height - object.trimmed_dimensions.height;
        let max_col = self.pitch.dimensions.width - object.trimmed_dimensions.width;

        for row in object.trim_offset.0..=max_row {
            for col in object.trim_offset.1..=max_col {
                // Check if placement at this position is valid
                if let Some(valid_move) = self.validate_placement(&object, Position { row, col }) {
                    valid_placements.push(valid_move);
                }
            }
        }

        // No valid moves found
        if valid_placements.is_empty() {
            return (0, 0);
        }

        // Evaluate all valid placements and pick the best one
        let best_move = evaluate_all_placements(
            &self.pitch,
            valid_placements,
            opponent_center,
            self.turn_number,
            self.our_contestant.symbols,
            &self.placement_history,
        );

        // Update game state
        self.placement_history.push(object);
        self.our_contestant.cells_captured += 1;

        // Convert from trimmed position to original object coordinates
        (
            best_move.position.col as i32 - best_move.object.trim_offset.1 as i32,
            best_move.position.row as i32 - best_move.object.trim_offset.0 as i32,
        )
    }

    /// Validates if a object can be placed at the given position
    /// Returns Some(PotentialMove) if valid, None otherwise
    ///
    /// Rules:
    /// - Object must overlap exactly ONE of our existing cells
    /// - Object cannot overlap any opponent cells
    /// - All object cells must fit on the pitch
    fn validate_placement(&self, object: &Object, position: Position) -> Option<PotentialMove> {
        let mut overlap_count = 0;

        // Check each filled cell in the object
        for (object_row, row_data) in object.trimmed_cells.iter().enumerate() {
            for (object_col, &object_cell) in row_data.iter().enumerate() {
                // Skip empty cells in the object
                if object_cell == '.' {
                    continue;
                }

                // Calculate pitch position for this object cell
                let pitch_row = object_row + position.row;
                let pitch_col = object_col + position.col;

                let pitch_cell = self.pitch.cells[pitch_row][pitch_col];

                // Check if overlapping with our own object (robot)
                if self.our_contestant.owns_cell(&pitch_cell) {
                    overlap_count += 1;
                    // Invalid: can only overlap exactly 1 cell
                    if overlap_count > 1 {
                        return None;
                    }
                }
                // Check if overlapping with opponent robot
                else if self.opponent.owns_cell(&pitch_cell) {
                    return None; // Invalid: cannot overlap opponent
                }
            }
        }

        // Must overlap exactly 1 of our cells
        if overlap_count != 1 {
            return None;
        }

        // Calculate strategic score for this placement
        let mut total_score = 0;
        for (object_row, row_data) in object.trimmed_cells.iter().enumerate() {
            for (object_col, &object_cell) in row_data.iter().enumerate() {
                let pitch_pos = Position {
                    row: position.row + object_row,
                    col: position.col + object_col,
                };
                total_score += self.score_cell_placement(object_cell, pitch_pos);
            }
        }

        Some(PotentialMove {
            position,
            score: total_score,
            object: object.clone(),
        })
    }

    /// Calculates strategic value of placing a object cell at a position
    /// Higher scores indicate better strategic positions
    fn score_cell_placement(&self, object_cell: char, pitch_position: Position) -> i32 {
        let is_filled_cell = object_cell == 'O';

        // Get cells adjacent to this position (up, down, left, right)
        let adjacent = get_adjacent_cells(&self.pitch, &pitch_position);

        if !is_filled_cell {
            // Empty cell in object - check the Pitch
            let current_cell = self.pitch.cells[pitch_position.row][pitch_position.col];

            if self.our_contestant.owns_cell(&current_cell) {
                1 // Slightly good - we're near our territory
            } else if self.opponent.owns_cell(&current_cell) {
                2 // Better - we're near opponent (good for blocking)
            } else {
                0 // Neutral empty space
            }
        } else {
            // Filled cell in object - prioritize placing next to opponent
            // This helps us "chase" and block them

            if adjacent.up.is_some() && self.opponent.owns_cell(&adjacent.up.unwrap()) {
                return 4; // High value - blocking opponent from above
            }
            if adjacent.down.is_some() && self.opponent.owns_cell(&adjacent.down.unwrap()) {
                return 4; // High value - blocking opponent from below
            }
            if adjacent.left.is_some() && self.opponent.owns_cell(&adjacent.left.unwrap()) {
                return 4; // High value - blocking opponent from left
            }
            if adjacent.right.is_some() && self.opponent.owns_cell(&adjacent.right.unwrap()) {
                return 4; // High value - blocking opponent from right
            }

            0 // No immediate tactical advantage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid_matrix::Dimensions;

    fn create_test_game() -> GameAnalysis {
        let contestant = Contestant {
            number: 1,
            symbols: ('a', '@'),
            cells_captured: 0,
        };
        let opponent = Contestant {
            number: 2,
            symbols: ('s', '$'),
            cells_captured: 0,
        };

        let pitch = Pitch {
            dimensions: Dimensions {
                width: 4,
                height: 4,
            },
            cells: vec![
                vec!['.', 'a', '.', '.'], // Our robot at (0,1)
                vec!['.', '.', '.', '.'],
                vec!['.', '.', 's', '.'], // Opponent at (2,2)
                vec!['.', '.', '.', '.'],
            ],
        };

        GameAnalysis::new(contestant, opponent, pitch)
    }

    fn create_test_object() -> Object {
        Object {
            original_dimensions: Dimensions {
                width: 2,
                height: 2,
            },
            original_cells: vec![vec!['O', '.'], vec!['.', 'O']],
            trimmed_dimensions: Dimensions {
                width: 2,
                height: 2,
            },
            trimmed_cells: vec![vec!['O', '.'], vec!['.', 'O']],
            filled_cell_count: 2,
            trim_offset: (0, 0),
        }
    }

    #[test]
    fn test_valid_placement() {
        let game = create_test_game();
        let object = create_test_object();

        // Place at (1,0) so object overlaps our 'a' at (0,1)
        let result = game.validate_placement(&object, Position { row: 0, col: 1 });

        assert!(result.is_some());
        let placement = result.unwrap();
        assert_eq!(placement.position.col, 1);
        assert_eq!(placement.position.row, 0);
    }

    #[test]
    fn test_invalid_placement_overlaps_opponent() {
        let game = create_test_game();
        let object = create_test_object();

        // Try to place where it overlaps opponent at (2,2)
        let result = game.validate_placement(&object, Position { row: 1, col: 1 });

        assert!(result.is_none()); // Should be invalid
    }
}
