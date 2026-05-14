// contestant.rs

/// Represents one robot/contestant in the game
#[derive(Debug, Clone, Copy)]
pub struct Contestant {
    pub number: u8,
    pub symbols: (char, char),
    pub cells_captured: usize,
}

impl Contestant {
    /// Checks whether a pitch cell belongs to this contestant
    pub fn owns_cell(&self, cell: &char) -> bool {
        *cell == self.symbols.0 || *cell == self.symbols.1
    }
}

/// Creates both contestants from engine input
/// Example input:
/// "$$$ exec p1 : [./filler]"
pub fn create_contestants(input: &str) -> (Contestant, Contestant) {
    let our_number = if input.contains("p1") { 1 } else { 2 };

    let our_contestant = if our_number == 1 {
        Contestant {
            number: 1,
            symbols: ('a', '@'),
            cells_captured: 0,
        }
    } else {
        Contestant {
            number: 2,
            symbols: ('s', '$'),
            cells_captured: 0,
        }
    };

    let opponent = if our_number == 1 {
        Contestant {
            number: 2,
            symbols: ('s', '$'),
            cells_captured: 0,
        }
    } else {
        Contestant {
            number: 1,
            symbols: ('a', '@'),
            cells_captured: 0,
        }
    };

    (our_contestant, opponent)
}