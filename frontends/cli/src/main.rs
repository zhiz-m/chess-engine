use chess_engine_core::{ChessEngine, PlayerState, GameState, Player, evaluate};

fn main() {
    // let white_state = PlayerState::new_from_state([
    //     vec!["a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2"],
    //     vec!["b1", "g1"],
    //     vec!["c1", "f1"],
    //     vec!["a1", "h1"],
    //     vec!["d1"],
    //     vec!["e1"],
    // ]);
    // let black_state = PlayerState::new_from_state([
    //     vec!["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7"],
    //     vec!["b8", "g8"],
    //     vec!["c8", "f8"],
    //     vec!["a8", "h8"],
    //     vec!["d8"],
    //     vec!["e8"],
    // ]);
    let white_state = PlayerState::new_from_state([
        vec!["a3", "b2", "c3", "d3", "f2", "g3", "h2"],
        vec!["g5"],
        vec!["g2", "h6"],
        vec!["e1", "e6"],
        vec!["f4"],
        vec!["g1"],
    ]);
    let black_state = PlayerState::new_from_state([
        vec!["a7", "b5", "c5", "d4", "f5", "g6", "h7"],
        vec!["c6"],
        vec!["h8", "b7"],
        vec!["d7", "c8"],
        vec!["d8"],
        vec!["g8"],
    ]);
    let mut game_state = 
    GameState::new_from_state(white_state, black_state, Player::White);
    println!("Initial score: {}", evaluate(&game_state));
    let mut engine = ChessEngine::<8, 8, 24>::new(42);
    println!("best value: {}", engine.solve(&mut game_state));

    engine.get_moves();
}
