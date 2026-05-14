// object.rs

use crate::grid_matrix::{Dimensions, Grid};
use std::io::Error;

/// Represents a game obect that needs to be placed on the pitch
/// Objects can have irregular shapes with 'O' marking filled cells
#[derive(Debug, Clone)]
pub struct Object {
    pub original_dimensions: Dimensions,
    pub original_cells: Vec<Vec<char>>,

    // After trimming empty edges
    pub trimmed_dimensions: Dimensions,
    pub trimmed_cells: Vec<Vec<char>>,

    pub filled_cell_count: usize,
    pub trim_offset: (usize, usize), // (row_offset, col_offset)
}

impl Object {
    /// Creates a new object from the header line
    /// Header format: "Object HEIGHT WIDTH:"
    pub fn from_header(header: &str) -> Self {
        // Parse: "Object 4 3:" -> width=4, height=3
        let parts: Vec<&str> = header.trim_end_matches(':').split_whitespace().collect();

        let width: usize = parts[1].parse().expect("Invalid object width");
        let height: usize = parts[2].parse().expect("Invalid object height");

        // Initialize with empty cells
        let cells = vec![vec!['.'; width]; height];

        Object {
            original_dimensions: Dimensions { width, height },
            original_cells: cells,
            trimmed_dimensions: Dimensions {
                width: 0,
                height: 0,
            },
            trimmed_cells: vec![],
            filled_cell_count: 0,
            trim_offset: (0, 0),
        }
    }

    /// Removes empty rows and columns from the edges
    /// This optimization makes placement checking faster
    fn trim_empty_edges(&mut self) {
        let mut top = 0;
        let mut bottom = self.height();
        let mut left = 0;
        let mut right = self.width();

        // Find top edge - skip empty rows from top
        while top < bottom && self.original_cells[top].iter().all(|&c| c == '.') {
            top += 1;
        }

        // Find bottom edge - skip empty rows from bottom
        while bottom > top && self.original_cells[bottom - 1].iter().all(|&c| c == '.') {
            bottom -= 1;
        }

        // Find left edge - skip empty columns from left
        while left < right && (top..bottom).all(|i| self.original_cells[i][left] == '.') {
            left += 1;
        }

        // Find right edge - skip empty columns from right
        while right > left && (top..bottom).all(|i| self.original_cells[i][right - 1] == '.') {
            right -= 1;
        }

        // Extract trimmed object
        let mut trimmed = vec![];
        for row_index in top..bottom {
            trimmed.push(self.original_cells[row_index][left..right].to_vec());
        }

        self.trimmed_cells = trimmed;
        self.trimmed_dimensions = Dimensions {
            width: self.trimmed_cells[0].len(),
            height: self.trimmed_cells.len(),
        };
        self.trim_offset = (top, left);
    }

    /// Loads the object shape from input stream
    /// Reads HEIGHT lines containing the object pattern
    pub fn load_shape<I: Iterator<Item = Result<String, Error>>>(&mut self, lines: &mut I) {
        for row_index in 0..self.height() {
            let line = match lines.next() {
                Some(Ok(l)) => l.trim_end().to_string(),
                _ => panic!("Unexpected EOF reading object row {}", row_index),
            };

            // Validate line length
            if line.chars().count() < self.width() {
                panic!(
                    "Invalid object row {}: expected {} chars, got {}",
                    row_index,
                    self.width(),
                    line.chars().count()
                );
            }

            // Extract cell data
            let row_data: Vec<char> = line.chars().take(self.width()).collect();

            if row_data.len() != self.width() {
                panic!(
                    "Object row {} has {} columns, expected {}",
                    row_index,
                    row_data.len(),
                    self.width()
                );
            }

            // Count filled cells (non-dots)
            self.filled_cell_count += row_data.iter().filter(|&&ch| ch != '.').count();

            self.original_cells[row_index] = row_data;
        }

        // Optimize by removing empty edges
        self.trim_empty_edges();
    }
}

impl Grid for Object {
    fn height(&self) -> usize {
        self.original_dimensions.height
    }
    fn width(&self) -> usize {
        self.original_dimensions.width
    }
    fn cells(&self) -> &Vec<Vec<char>> {
        &self.original_cells
    }
    fn cells_mut(&mut self) -> &mut Vec<Vec<char>> {
        &mut self.original_cells
    }
}

use std::fmt;

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_with_padding() {
        let mut object = Object {
            original_dimensions: Dimensions {
                width: 5,
                height: 5,
            },
            original_cells: vec![
                vec!['.', '.', '.', '.', '.'], // Empty top
                vec!['.', 'O', 'O', '.', '.'], // Content
                vec!['.', 'O', '.', '.', '.'], // Content
                vec!['.', '.', '.', '.', '.'], // Empty bottom
                vec!['.', '.', '.', '.', '.'], // Empty bottom
            ],
            trimmed_dimensions: Dimensions {
                width: 0,
                height: 0,
            },
            trimmed_cells: vec![],
            filled_cell_count: 0,
            trim_offset: (0, 0),
        };

        object.trim_empty_edges();

        // Should trim to 2x2 content area
        assert_eq!(object.trimmed_dimensions.width, 2);
        assert_eq!(object.trimmed_dimensions.height, 2);
        assert_eq!(object.trim_offset, (1, 1));
        assert_eq!(object.trimmed_cells, vec![vec!['O', 'O'], vec!['O', '.']]);
    }

    #[test]
    fn test_trim_no_padding() {
        let mut object = Object {
            original_dimensions: Dimensions {
                width: 2,
                height: 2,
            },
            original_cells: vec![vec!['O', '.'], vec!['.', 'O']],
            trimmed_dimensions: Dimensions {
                width: 0,
                height: 0,
            },
            trimmed_cells: vec![],
            filled_cell_count: 0,
            trim_offset: (0, 0),
        };

        object.trim_empty_edges();

        // Should remain same size
        assert_eq!(object.trimmed_dimensions.width, 2);
        assert_eq!(object.trimmed_dimensions.height, 2);
        assert_eq!(object.trim_offset, (0, 0));
    }
}
