use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn test_game_replay_map01() {
    // Sample input - you'll replace this with actual captured game data
    let game_input = r#"$$$ exec p1 : [solution/filler]
Anfield 15 17:
000 .................
001 .................
002 .................
003 .................
004 .................
005 .................
006 .................
007 .................
008 @................
009 .................
010 .................
011 .................
012 .................
013 .................
014 .................
Piece 3 4:
....
.##.
.##.
Anfield 15 17:
000 .................
001 .................
002 .................
003 .................
004 .................
005 .................
006 .................
007 .................
008 @................
009 .................
010 .................
011 .................
012 .................
013 .................
014 .................
Piece 2 2:
.#
#.
"#;

    // Run your compiled bot
    let mut child = Command::new("./target/debug/filler")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn filler process");

    // Send input
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(game_input.as_bytes()).expect("Failed to write to stdin");
    }

    // Read output
    let output = child.wait_with_output().expect("Failed to wait on child");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let moves: Vec<&str> = stdout.lines().collect();

    // Verify expected moves
    assert!(moves.len() >= 2, "Expected at least 2 moves");
    
    // Check first move is valid format
    let first_move: Vec<&str> = moves[0].split_whitespace().collect();
    assert_eq!(first_move.len(), 2, "Move should be 'X Y' format");
    
    // Parse coordinates
    let x: i32 = first_move[0].parse().expect("X should be a number");
    let y: i32 = first_move[1].parse().expect("Y should be a number");
    
    // Basic sanity checks
    assert!(x >= 0 && x < 17, "X coordinate out of bounds");
    assert!(y >= 0 && y < 15, "Y coordinate out of bounds");
    
    println!("First move: {} {}", x, y);
}

#[test]
fn test_no_valid_placement() {
    // Test case where bot should return "0 0"
    let game_input = r#"$$$ exec p1 : [solution/filler]
Anfield 3 3:
000 @$$
001 $$$
002 $$$
Piece 2 2:
##
##
"#;

    let mut child = Command::new("./target/debug/filler")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(game_input.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert_eq!(stdout.trim(), "0 0", "Should return 0 0 when no valid placement");
}