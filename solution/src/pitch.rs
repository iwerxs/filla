// pitch.rs

use crate::grid_matrix::{Dimensions, Grid};
use std::io::Error;

/// State of the game pitch
/// Tracks all placed pieces (objects) and empty spaces
#[derive(Debug, Clone)]
pub struct Pitch {
    pub dimensions: Dimensions,
    pub cells: Vec<Vec<char>>,
}

/// Represents a position on the pitch
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Pitch {
    /// Creates a new empty pitch from the header line
    /// Header format: "Anfield HEIGHT WIDTH:"
    pub fn new(header: &str) -> Self {
        // Parse: "Anfield 40 30:" -> width=40, height=30
        let parts: Vec<&str> = header.trim_end_matches(':').split_whitespace().collect();

        let width: usize = parts[1].parse().expect("Invalid pitch width");
        let height: usize = parts[2].parse().expect("Invalid pitch height");

        // Initialize with empty cells '.'
        let cells = vec![vec!['.'; width]; height];

        Pitch {
            dimensions: Dimensions { height, width },
            cells,
        }
    }

    /// Updates state of pitch from input
    /// Reads HEIGHT lines, each containing the row state
    pub fn load_state<I: Iterator<Item = Result<String, Error>>>(&mut self, lines: &mut I) {
        // Skip the column number header line (e.g., "    01234567...")
        let _ = lines.next();

        // Read each row of the pitch
        for row_index in 0..self.height() {
            let line = match lines.next() {
                Some(Ok(l)) => l.trim_end().to_string(),
                _ => panic!("Unexpected EOF reading pitch row {}", row_index),
            };

            // Validate line length
            // Format: "000 .a.@..." (3 digit row number + space + cells)
            if line.chars().count() < 4 + self.width() {
                panic!(
                    "Invalid row {}: expected {} chars, got {}",
                    row_index,
                    4 + self.width(),
                    line.chars().count()
                );
            }

            // Extract cell data (skip first 4 characters: "000 ")
            let row_data: Vec<char> = line[4..].chars().take(self.width()).collect();

            // Verify we got the right number of cells
            if row_data.len() != self.width() {
                panic!(
                    "Row {} has {} columns, expected {}",
                    row_index,
                    row_data.len(),
                    self.width()
                );
            }

            // Update state of Pitch
            self.cells_mut()[row_index] = row_data;
        }
    }
}

impl Grid for Pitch {
    fn height(&self) -> usize {
        self.dimensions.height
    }

    fn width(&self) -> usize {
        self.dimensions.width
    }

    fn cells(&self) -> &Vec<Vec<char>> {
        &self.cells
    }

    fn cells_mut(&mut self) -> &mut Vec<Vec<char>> {
        &mut self.cells
    }
}

use std::fmt;

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}
