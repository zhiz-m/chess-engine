use std::{time::{SystemTime, Duration}, fs::File, io::Read, thread};

use chess_engine_core::{ChessEngine, GameState, evaluate};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    #[clap(long, short)]
    perft: Option<usize>,

    #[clap(long, short='t')]
    allowed_time: Option<u64>,

    #[clap(long,short='s')]
    path_to_state: Option<String>,

    #[clap(long,short='d')]
    fixed_depth: Option<usize>,
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
    let mut engine = ChessEngine::new(10, 40, 42);

    if let Some(depth) = cli.fixed_depth{
        engine.solve(&game_state, depth);
        engine.get_result();    
        return;
    }

    let max_duration = Duration::from_secs(cli.allowed_time.unwrap_or(3));
    thread::spawn(move ||{
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
