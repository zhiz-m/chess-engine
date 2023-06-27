use std::{time::{SystemTime, Duration}, fs::File, io::Read, thread};

use chess_engine_core::{ChessEngine, GameState, evaluate};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    #[clap(long, short)]
    perft: Option<usize>,

    #[clap(long, short)]
    allowed_time: Option<u64>,

    #[clap(long,short='s')]
    path_to_state: Option<String>
}

fn main() {    
    let cli = Cli::parse();
    if let Some(perft_depth) = cli.perft{
        println!("perft of depth {} from starting position", perft_depth);
        
        // println!("Initial score: {}", evaluate(&game_state));

        let mut now = SystemTime::now();

        let mut engine = ChessEngine::new(perft_depth, 40, 42);

        for i in 1..=perft_depth{
            let mut game_state = GameState::default();

            // let mut engine = ChessEngine::new(i, 40, 42);
            
            println!("{}: {} nodes", i, engine.perft(&mut game_state, i));
            let cur_time = SystemTime::now();
            println!("{} ms", cur_time.duration_since(now).unwrap().as_millis());
            now = cur_time;            
        }

        return;        
    }
    // let white_state = PlayerState::new_from_state(vec![
    //     vec!["a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2"],
    //     vec!["b1", "g1"],
    //     vec!["c1", "f1"],
    //     vec!["a1", "h1"],
    //     vec!["d1"],
    //     vec!["e1"],
    // ]).unwrap();
    // let black_state = PlayerState::new_from_state(vec![
    //     vec!["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7"],
    //     vec!["b8", "g8"],
    //     vec!["c8", "f8"],
    //     vec!["a8", "h8"],
    //     vec!["d8"],
    //     vec!["e8"],
    // ]).unwrap();
    // let white_state = PlayerState::new_from_state([
    //     vec!["a3", "b2", "c3", "d3", "f2", "g3", "h2"],
    //     vec!["g5"],
    //     vec!["g2", "h6"],
    //     vec!["e1", "e6"],
    //     vec!["f4"],
    //     vec!["g1"],
    // ]);
    // let black_state = PlayerState::new_from_state([
    //     vec!["a7", "b5", "c5", "d4", "f5", "g6", "h7"],
    //     vec!["c6"],
    //     vec!["h8", "b7"],
    //     vec!["d7", "c8"],
    //     vec!["d8"],
    //     vec!["g8"],
    // ]);

    let max_duration = Duration::from_secs(cli.allowed_time.unwrap_or(3));
    let game_state = match cli.path_to_state{
        Some(path) => {
            let mut buf = String::new();
            let mut file = File::open(&path).unwrap();
            file.read_to_string(&mut buf).unwrap();

            serde_json::from_str(&buf).unwrap()
        },
        None => GameState::default(),
    };

    // let mut game_state = 
    // GameState::new_from_state(white_state, black_state, Player::White);
    println!("Initial score: {}", evaluate(&game_state));
    // let start_time = SystemTime::now();

    // let (rx,tx) = mpsc::channel();

    thread::spawn(move ||{
        let mut engine = ChessEngine::new(10, 40, 42);
        for i in 1..10{
            // rx.send((i,engine.solve(&game_state)));
            println!("depth: {}", i);
            engine.solve(&game_state, i);
            engine.get_result();
            println!();
        }
    });        

    thread::sleep(max_duration);

    // println!("best value: {}, time: {}ms", engine.solve(&game_state), SystemTime::now().duration_since(start_time).unwrap().as_millis());

    // engine.get_result();
}
