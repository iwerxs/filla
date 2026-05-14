// grid_matrix.rs

/// Core trait defining grid-like structures (pitches and objects)
/// Provides common functionality for anything with rows and columns

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

/// Common interface for grid-based structures
/// Both the game pitch and objects implement this trait
pub trait Grid {
    fn height(&self) -> usize;
    fn width(&self) -> usize;
    fn cells(&self) -> &Vec<Vec<char>>;
    fn cells_mut(&mut self) -> &mut Vec<Vec<char>>;

    /// Debug helper - prints grid to string format
    fn to_display_string(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "Grid ({} rows x {} cols):\n",
            self.height(),
            self.width()
        ));
        for row in self.cells() {
            for &ch in row {
                output.push(ch);
            }
            output.push('\n');
        }
        output
    }
}
