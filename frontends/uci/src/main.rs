use chess_engine_core::{
    canonical_to_pos, pos_to_coord, ChessEngine, GameState, Move, Piece, Player,
};
use std::{
    io::{self, BufRead, Write},
    time::SystemTime,
};
use vampirc_uci::{
    parse_one, UciInfoAttribute, UciMessage, UciMove, UciOptionConfig, UciPiece, UciSearchControl,
    UciSquare,
};

fn log_unnormalized_message(message: &UciMessage) {
    println!(
        "{}",
        UciMessage::Info(vec![UciInfoAttribute::Any(
            "unnormalized-message".to_owned(),
            format!("{}", message)
        )])
    );
}

fn log_normalized_message(message: &UciMessage) {
    println!(
        "{}",
        UciMessage::Info(vec![UciInfoAttribute::Any(
            "got-message".to_owned(),
            format!("{}", message)
        )])
    );
}

fn piece_uci_to_engine(uci_piece: UciPiece) -> Piece {
    match uci_piece {
        UciPiece::Pawn => Piece::Pawn,
        UciPiece::Knight => Piece::Knight,
        UciPiece::Bishop => Piece::Bishop,
        UciPiece::Rook => Piece::Rook,
        UciPiece::Queen => Piece::Queen,
        UciPiece::King => Piece::King,
    }
}

fn piece_engine_to_uci(piece: Piece) -> UciPiece {
    match piece {
        Piece::Pawn => UciPiece::Pawn,
        Piece::Knight => UciPiece::Knight,
        Piece::Bishop => UciPiece::Bishop,
        Piece::Rook => UciPiece::Rook,
        Piece::Queen => UciPiece::Queen,
        Piece::King => UciPiece::King,
    }
}

fn pos_engine_to_uci(pos: u8) -> UciSquare {
    let tup = pos_to_coord(pos);
    UciSquare {
        file: (7 - tup.1 + 'a' as i8) as u8 as char,
        rank: tup.0 as u8 + 1,
    }
}

fn move_engine_to_uci(player: Player, mov: Move) -> UciMove {
    let (prev_pos, new_pos, promotion) = match mov {
        Move::Move {
            prev_pos, new_pos, ..
        } => (prev_pos, new_pos, None),
        Move::Castle { is_short } => {
            let (prev_pos, new_pos) = match (player, is_short) {
                (Player::White, true) => (3, 1),
                (Player::White, false) => (3, 5),
                (Player::Black, true) => (59, 57),
                (Player::Black, false) => (59, 61),
            };
            (prev_pos, new_pos, None)
        }
        Move::PawnPromote {
            prev_pos,
            new_pos,
            pieces,
            ..
        } => {
            let (promoted_to_piece, _) = pieces.to_square_types();
            (
                prev_pos,
                new_pos,
                Some(piece_engine_to_uci(
                    promoted_to_piece.to_piece_for_io().expect("cannot be none"),
                )),
            )
        }
        Move::EnPassant {
            prev_column,
            new_column,
        } => match player {
            Player::White => (32 + prev_column, 40 + new_column, None),
            Player::Black => (24 + prev_column, 16 + new_column, None),
        },
    };
    UciMove {
        from: pos_engine_to_uci(prev_pos),
        to: pos_engine_to_uci(new_pos),
        promotion,
    }
}

const NAME: &'static str = "loglogn-bot";
const AUTHOR: &'static str = "loglogn";

fn main() {
    let mut game_state = GameState::default();
    let mut engine = ChessEngine::new(16, 40, 13);
    for line in io::stdin().lock().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());

        match msg {
            UciMessage::Uci => {
                println!(
                    "{}",
                    UciMessage::Id {
                        name: Some(NAME.to_owned()),
                        author: Some(AUTHOR.to_owned())
                    }
                );
                println!("{}", UciMessage::UciOk);
            }
            UciMessage::Debug(_) => log_unnormalized_message(&msg),
            UciMessage::IsReady => println!("{}", UciMessage::ReadyOk),
            UciMessage::Register { .. } => log_unnormalized_message(&msg),
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                engine.clear_move_history_threefold_repetition();
                game_state = if startpos {
                    GameState::new_with_hash(&engine.zoborist_state)
                } else {
                    GameState::new_from_fen(fen.unwrap().as_str(), &engine.zoborist_state).unwrap()
                };
                for UciMove {
                    from,
                    to,
                    promotion,
                } in moves
                {
                    let to_u8 = |s: UciSquare| canonical_to_pos(&format!("{}{}", s.file, s.rank));
                    let promoted_to_piece = promotion.map(piece_uci_to_engine);
                    engine
                        .make_move_raw_parts(
                            &mut game_state,
                            to_u8(from),
                            to_u8(to),
                            promoted_to_piece,
                        )
                        .unwrap();
                }
            }
            UciMessage::SetOption { .. } => log_unnormalized_message(&msg),
            UciMessage::UciNewGame => log_unnormalized_message(&msg),
            UciMessage::Stop => log_unnormalized_message(&msg),
            UciMessage::PonderHit => log_unnormalized_message(&msg),
            UciMessage::Quit => break,
            UciMessage::Go {
                time_control: _,
                search_control,
            } => {
                // let max_duration = Duration::from_secs(3);
                // thread::scope(|s|{
                //     let handle = s.spawn(|| {
                //         for i in 1..10 {
                //             // rx.send((i,engine.solve(&game_state)));
                //             println!("depth: {}", i);
                //             engine.solve(&game_state, i);
                //             engine.get_result();
                //             println!();
                //         }
                //     });
                //     thread::sleep(max_duration);
                //     let mov = handle.join();
                // });
                let start = SystemTime::now();

                let depth = (|| {
                    let search_control = search_control?;
                    let depth = search_control.depth?;
                    Some(depth)
                })()
                .unwrap_or(10);
                let mut mov = Move::Castle { is_short: false };
                let mut score = 0;
                for i in ((2 - (depth as usize % 2))..=(depth as usize)).step_by(2) {
                    println!(
                        "{}",
                        UciMessage::Info(vec![UciInfoAttribute::Any(
                            "current depth".to_owned(),
                            format!("{}", i)
                        )])
                    );
                    let start = SystemTime::now();
                    score = engine.solve(&game_state, i);
                    mov = engine.get_best_calculated_move(game_state.player).unwrap();
                    println!(
                        "{}",
                        UciMessage::Info(vec![UciInfoAttribute::Any(
                            "iterative search finished".to_owned(),
                            format!(
                                "depth {}, score {}, {}, {}ms",
                                i,
                                score,
                                mov,
                                SystemTime::now().duration_since(start).unwrap().as_millis()
                            )
                        )])
                    );

                    engine.lift_killer_moves(2);
                }
                println!(
                    "{}",
                    UciMessage::Info(vec![UciInfoAttribute::Any(
                        "best score".to_owned(),
                        format!("{}", score)
                    )])
                );
                // let mov = engine.get_best_calculated_move(game_state.player).unwrap();
                println!(
                    "{}",
                    UciMessage::Info(vec![UciInfoAttribute::Any(
                        "best move".to_owned(),
                        format!("{}", mov,)
                    )])
                );
                println!(
                    "{}",
                    UciMessage::Info(vec![UciInfoAttribute::Any(
                        "time".to_owned(),
                        format!(
                            "{} ms",
                            SystemTime::now().duration_since(start).unwrap().as_millis()
                        )
                    )])
                );
                println!(
                    "{}",
                    UciMessage::BestMove {
                        best_move: move_engine_to_uci(game_state.player, mov),
                        ponder: None
                    }
                );
                engine.print_debug()
            }

            UciMessage::Unknown(msg, _) => {
                if msg.to_lowercase().starts_with("go perft") {
                    let depth = str::parse::<usize>(&msg[9..]).unwrap_or(8);
                    let mut engine = ChessEngine::new(depth, 40, 42);
                    for depth in 1..=depth {
                        let start = SystemTime::now();
                        println!(
                            "{}",
                            UciMessage::Info(vec![UciInfoAttribute::Any(
                                "Perft".to_owned(),
                                format!(
                                    "depth {}, value {}, {} ms",
                                    depth,
                                    engine.perft(&mut game_state, depth),
                                    SystemTime::now().duration_since(start).unwrap().as_millis()
                                )
                            )])
                        );
                    }
                } else {
                    println!(
                        "{}",
                        UciMessage::Info(vec![UciInfoAttribute::Any(
                            "Unexpected message".to_owned(),
                            format!("{}", msg)
                        )])
                    );
                }
            }

            // why are these here?
            UciMessage::Id { .. }
            | UciMessage::UciOk
            | UciMessage::ReadyOk
            | UciMessage::BestMove { .. }
            | UciMessage::CopyProtection(_)
            | UciMessage::Registration(_)
            | UciMessage::Option(_)
            | UciMessage::Info(_) => println!(
                "{}",
                UciMessage::Info(vec![UciInfoAttribute::Any(
                    "Unexpected message".to_owned(),
                    format!("{}", msg)
                )])
            ),
        }

        io::stdout().flush().unwrap();
    }
}
