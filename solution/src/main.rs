// main.rs

mod pitch;
mod game_analysis;
mod grid_matrix;
mod object;
mod contestant;
mod tactics;

use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut input_lines = stdin.lock().lines();

    // Read contestant assignment only
    let contestant_assignment = match input_lines.next() {
        Some(Ok(line)) => line,
        _ => return,
    };

    let (our_contestant, opponent) =
        contestant::create_contestants(&contestant_assignment);

    // Wait for first Anfield line
    let mut game: Option<game_analysis::GameAnalysis> = None;

    loop {
        let line = match input_lines.next() {
            Some(Ok(l)) => l,
            _ => break,
        };

        // Read pitch state
        if line.starts_with("Anfield") {
            let pitch_dimensions = line.clone();

            let pitch_instance = pitch::Pitch::new(&pitch_dimensions);

            if game.is_none() {
                game = Some(game_analysis::GameAnalysis::new(
                    our_contestant,
                    opponent,
                    pitch_instance,
                ));
            }

            if let Some(ref mut g) = game {
                g.pitch.load_state(&mut input_lines);
            }
        }

        // Read object state
        if line.starts_with("Piece") {
            let mut object_to_place = object::Object::from_header(&line);

            object_to_place.load_shape(&mut input_lines);

            if let Some(ref mut g) = game {
                let (best_col, best_row) =
                    g.calculate_best_move(object_to_place);

                println!("{} {}", best_col, best_row);
            } else {
                println!("0 0");
            }
        }
    }
}