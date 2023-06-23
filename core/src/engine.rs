use std::{collections::BTreeSet, cmp::Reverse};

use crate::{types::{Move, ValueMovePair, Piece}, game_data::ZoboristState, config::MOVE_TABLE_SIZE, Player, GameState, util, move_table::{MoveTable, MoveEntry}, eval, killer_table::KillerTable, move_buffer::MoveBuffer};

pub struct ChessEngine<
    const MAX_DEPTH_SOFT: usize,
    const MAX_DEPTH_CAPTURE: usize,
    const MAX_DEPTH_HARD: usize
> {
    // state: GameState,
    move_buf: MoveBuffer<MAX_DEPTH_HARD>,
    killer_table: KillerTable<MAX_DEPTH_HARD>,
    opp_move_grid: u64,
    nodes_explored: u64,
    cache_hits: u64,
    calculated_moves: BTreeSet<ValueMovePair>,
    zoborist_state: ZoboristState,
    // state_cache: HashMap<HashType, (usize, i32), FxBuildHasher>
    state_cache: MoveTable<MOVE_TABLE_SIZE>
}

// impl<const MAX_DEPTH_SOFT: usize, const MAX_DEPTH_CAPTURE: usize, const MAX_DEPTH_HARD: usize>
//     Default for ChessEngine<MAX_DEPTH_SOFT, MAX_DEPTH_CAPTURE, MAX_DEPTH_HARD>
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl<const MAX_DEPTH_SOFT: usize, const MAX_DEPTH_CAPTURE: usize, const MAX_DEPTH_HARD: usize>
    ChessEngine<MAX_DEPTH_SOFT, MAX_DEPTH_CAPTURE, MAX_DEPTH_HARD>
{
    pub fn new(zoborist_state_seed: u64) -> Self {
        if MAX_DEPTH_SOFT > MAX_DEPTH_CAPTURE || MAX_DEPTH_CAPTURE > MAX_DEPTH_HARD {
            panic!("invalid depth parameters");
        }
        // const BUF: Vec<Move> = vec![];
        ChessEngine {
            opp_move_grid: 0,
            move_buf: Default::default(),
            killer_table: Default::default(),
            nodes_explored: 0,
            cache_hits: 0,
            calculated_moves: BTreeSet::default(),
            zoborist_state: ZoboristState::new(zoborist_state_seed),
            // state_cache: HashMap::with_hasher(FxBuildHasher::default())
            state_cache: Default::default()
        }
    }

    // // TYPE: whether to save it to teh move buffer or to put i
    // fn emit_move<const TYPE: bool>(&mut self, next_move: Move, player: Player, depth: usize) {
    //     if TYPE {
    //         if let Move::Move {
    //             prev_pos,
    //             new_pos,
    //             captured_piece,
    //             piece,
    //         } = next_move
    //         {
    //             if piece == Piece::Pawn
    //                 && ((player == Player::White && (new_pos >> 3) == 7)
    //                     || (player == Player::Black && (new_pos >> 3) == 0))
    //             {
    //                 const PIECES: [Piece; 4] =
    //                     [Piece::Queen, Piece::Knight, Piece::Rook, Piece::Bishop];
    //                 for promoted_to_piece in PIECES {
    //                     self.move_buf[depth].push(Move::PawnPromote {
    //                         prev_pos,
    //                         new_pos,
    //                         promoted_to_piece,
    //                         captured_piece,
    //                     });
    //                 }
    //                 return;
    //             }
    //         }
    //         self.move_buf[depth].push(next_move);
    //     } else if let Move::Move { new_pos, .. } = next_move {
    //         // opp move grid is for player squares that are under attack, is used to check if king can castle
    //         // note that we can count for pawn moves and pawn captures: this doesn't matter
    //         self.opp_move_grid |= 1u64 << new_pos;
    //     }
    // }

    // fn get_moves_by_deltas<const TYPE: bool, T: IntoIterator<Item = (i8, i8)>>(
    //     &mut self,
    //     state: &GameState,
    //     prev_pos: u8,
    //     player: Player,
    //     piece: Piece,
    //     deltas: T,
    //     depth: usize,
    // ) {
    //     for (dy, dx) in deltas {
    //         let mut res_coord = util::pos_to_coord(prev_pos);
    //         res_coord.0 += dy;
    //         res_coord.1 += dx;
    //         while res_coord.0 >= 0 && res_coord.0 < 8 && res_coord.1 >= 0 && res_coord.1 < 8 {
    //             let new_pos = util::coord_to_pos(res_coord);
    //             if state.get_state(player).square_occupied(new_pos) {
    //                 break;
    //             }
    //             // self.emit_move::<TYPE>(prev_pos, new_pos, piece, MoveType::MoveCapture, depth);
    //             // self.move_buf.push((pos, res_pos, MoveType::MoveCapture));
    //             if let Some(captured_piece) =
    //                 state.get_state(player.opp()).get_piece_at_pos(new_pos)
    //             {
    //                 self.emit_move::<TYPE>(
    //                     Move::Move {
    //                         prev_pos,
    //                         new_pos,
    //                         piece,
    //                         captured_piece: Some(captured_piece),
    //                     },
    //                     player,
    //                     depth,
    //                 );
    //                 break;
    //             }
    //             self.emit_move::<TYPE>(
    //                 Move::Move {
    //                     prev_pos,
    //                     new_pos,
    //                     piece,
    //                     captured_piece: None,
    //                 },
    //                 player,
    //                 depth,
    //             );
    //             res_coord.0 += dy;
    //             res_coord.1 += dx;
    //         }
    //     }
    // }

    // // todo: enpassant, double move
    // fn get_move_by_piece<const TYPE: bool>(
    //     &mut self,
    //     state: &GameState,
    //     piece: Piece,
    //     prev_pos: u8,
    //     player: Player,
    //     depth: usize,
    // ) {
    //     match piece {
    //         Piece::Pawn => {
    //             if player == Player::White {
    //                 if !state.get_state(player.opp()).square_occupied(prev_pos + 8) {
    //                     self.emit_move::<TYPE>(
    //                         Move::Move {
    //                             prev_pos,
    //                             new_pos: prev_pos + 8,
    //                             piece,
    //                             captured_piece: None,
    //                         },
    //                         player,
    //                         depth,
    //                     );
    //                     if (prev_pos >> 3) == 1
    //                         && !state.get_state(player.opp()).square_occupied(prev_pos + 16)
    //                     {
    //                         self.emit_move::<TYPE>(
    //                             Move::Move {
    //                                 prev_pos,
    //                                 new_pos: prev_pos + 16,
    //                                 piece,
    //                                 captured_piece: None,
    //                             },
    //                             player,
    //                             depth,
    //                         );
    //                     }
    //                 }

    //                 if (prev_pos & 0b111) > 0 {
    //                     if let Some(captured_piece) =
    //                         state.get_state(player.opp()).get_piece_at_pos(prev_pos + 7)
    //                     {
    //                         self.emit_move::<TYPE>(
    //                             Move::Move {
    //                                 prev_pos,
    //                                 new_pos: prev_pos + 7,
    //                                 piece,
    //                                 captured_piece: Some(captured_piece),
    //                             },
    //                             player,
    //                             depth,
    //                         );
    //                     }
    //                 }
    //                 if (prev_pos & 0b111) < 7 {
    //                     if let Some(captured_piece) =
    //                         state.get_state(player.opp()).get_piece_at_pos(prev_pos + 9)
    //                     {
    //                         self.emit_move::<TYPE>(
    //                             Move::Move {
    //                                 prev_pos,
    //                                 new_pos: prev_pos + 9,
    //                                 piece,
    //                                 captured_piece: Some(captured_piece),
    //                             },
    //                             player,
    //                             depth,
    //                         );
    //                     }
    //                 }
    //             } else {
    //                 if !state.get_state(player.opp()).square_occupied(prev_pos - 8) {
    //                     self.emit_move::<TYPE>(
    //                         Move::Move {
    //                             prev_pos,
    //                             new_pos: prev_pos - 8,
    //                             piece,
    //                             captured_piece: None,
    //                         },
    //                         player,
    //                         depth,
    //                     );
    //                     if (prev_pos >> 3) == 6
    //                         && !state.get_state(player.opp()).square_occupied(prev_pos - 16)
    //                     {
    //                         self.emit_move::<TYPE>(
    //                             Move::Move {
    //                                 prev_pos,
    //                                 new_pos: prev_pos - 16,
    //                                 piece,
    //                                 captured_piece: None,
    //                             },
    //                             player,
    //                             depth,
    //                         );
    //                     }
    //                 }
    //                 if (prev_pos & 0b111) > 0 {
    //                     if let Some(captured_piece) =
    //                         state.get_state(player.opp()).get_piece_at_pos(prev_pos - 9)
    //                     {
    //                         self.emit_move::<TYPE>(
    //                             Move::Move {
    //                                 prev_pos,
    //                                 new_pos: prev_pos - 9,
    //                                 piece,
    //                                 captured_piece: Some(captured_piece),
    //                             },
    //                             player,
    //                             depth,
    //                         );
    //                     }
    //                 }
    //                 if (prev_pos & 0b111) < 7 {
    //                     if let Some(captured_piece) =
    //                         state.get_state(player.opp()).get_piece_at_pos(prev_pos - 7)
    //                     {
    //                         self.emit_move::<TYPE>(
    //                             Move::Move {
    //                                 prev_pos,
    //                                 new_pos: prev_pos - 7,
    //                                 piece,
    //                                 captured_piece: Some(captured_piece),
    //                             },
    //                             player,
    //                             depth,
    //                         );
    //                     }
    //                 }
    //             }
    //         }
    //         Piece::Knight => {
    //             const KNIGHT_DELTAS: [(i8, i8); 8] = [
    //                 (-1, -2),
    //                 (-1, 2),
    //                 (1, -2),
    //                 (1, 2),
    //                 (-2, -1),
    //                 (-2, 1),
    //                 (2, -1),
    //                 (2, 1),
    //             ];
    //             let coord = util::pos_to_coord(prev_pos);
    //             for (dy, dx) in KNIGHT_DELTAS {
    //                 let res_coord = (coord.0 + dy, coord.1 + dx);
    //                 if res_coord.0 >= 0
    //                 && res_coord.0 < 8
    //                 && res_coord.1 >= 0
    //                 && res_coord.1 < 8
    //                 && !state.get_state(player).square_occupied(util::coord_to_pos(res_coord))
    //                 {
    //                     let new_pos = util::coord_to_pos(res_coord);
    //                     self.emit_move::<TYPE>(
    //                         Move::Move {
    //                             prev_pos,
    //                             new_pos,
    //                             piece,
    //                             captured_piece: state
    //                                 .get_state(player.opp())
    //                                 .get_piece_at_pos(new_pos),
    //                         },
    //                         player,
    //                         depth,
    //                     );
    //                 }
    //             }
    //         }
    //         Piece::Bishop => {
    //             const BISHOP_DELTAS: [(i8, i8); 4] = [(1, -1), (1, 1), (-1, -1), (-1, 1)];
    //             self.get_moves_by_deltas::<TYPE, _>(
    //                 state,
    //                 prev_pos,
    //                 player,
    //                 Piece::Bishop,
    //                 BISHOP_DELTAS,
    //                 depth,
    //             );
    //         }
    //         Piece::Rook => {
    //             const ROOK_DELTAS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, -1), (0, 1)];
    //             self.get_moves_by_deltas::<TYPE, _>(
    //                 state,
    //                 prev_pos,
    //                 player,
    //                 Piece::Rook,
    //                 ROOK_DELTAS,
    //                 depth,
    //             );
    //         }
    //         Piece::Queen => {
    //             const QUEEN_DELTAS: [(i8, i8); 8] = [
    //                 (1, -1),
    //                 (1, 1),
    //                 (-1, -1),
    //                 (-1, 1),
    //                 (1, 0),
    //                 (-1, 0),
    //                 (0, -1),
    //                 (0, 1),
    //             ];
    //             self.get_moves_by_deltas::<TYPE, _>(
    //                 state,
    //                 prev_pos,
    //                 player,
    //                 Piece::Queen,
    //                 QUEEN_DELTAS,
    //                 depth,
    //             );
    //         }
    //         Piece::King => {
    //             const KING_DELTAS: [(i8, i8); 8] = [
    //                 (1, -1),
    //                 (1, 1),
    //                 (-1, -1),
    //                 (-1, 1),
    //                 (1, 0),
    //                 (-1, 0),
    //                 (0, -1),
    //                 (0, 1),
    //             ];
    //             let coord = util::pos_to_coord(prev_pos);

    //             for (dy, dx) in KING_DELTAS {
    //                 let res_coord = (coord.0 + dy, coord.1 + dx);
    //                 let new_pos = util::coord_to_pos(res_coord);

    //                 // square is valid
    //                 // check for square under attack by opposing piece is done during tree search
    //                 if res_coord.0 >= 0
    //                     && res_coord.0 < 8
    //                     && res_coord.1 >= 0
    //                     && res_coord.1 < 8
    //                     && state.get_state(player).piece_grid & (1 << new_pos) == 0
    //                 {
    //                     self.emit_move::<TYPE>(
    //                         Move::Move {
    //                         prev_pos,
    //                             new_pos,
    //                             piece,
    //                             captured_piece: state
    //                                 .get_state(player.opp())
    //                                 .get_piece_at_pos(new_pos),
    //                         },
    //                         player,
    //                         depth,
    //                     );
    //                 }
    //             }

    //             let player_state = state.get_state(player);
    //             let offset = if player == Player::White { 0 } else { 7 };

    //             // optimization
    //             if !player_state.square_occupied(5 + offset)
    //                 && !player_state.square_occupied(6 + offset)
    //                 && !player_state.square_occupied(2 + offset)
    //                 && !player_state.square_occupied(3 + offset)
    //                 && !player_state.square_occupied(4 + offset)
    //                 && (player_state.meta.can_castle_short
    //                     && self.opp_move_grid
    //                         | (1 << (4 + offset))
    //                         | (1 << (5 + offset))
    //                         | (1 << (6 + offset))
    //                         == 0)
    //                 || (player_state.meta.can_castle_long
    //                     && self.opp_move_grid
    //                         | (1 << (4 + offset))
    //                         | (1 << (3 + offset))
    //                         | (1 << (2 + offset))
    //                         == 0)
    //             {
    //                 self.opp_move_grid = 0;
    //                 self.get_all_moves::<false>(state, state.player.opp(), depth);

    //                 if !player_state.square_occupied(5 + offset)
    //                     && !player_state.square_occupied(6 + offset)
    //                     && player_state.meta.can_castle_short
    //                     && self.opp_move_grid
    //                         | (1 << (4 + offset))
    //                         | (1 << (5 + offset))
    //                         | (1 << (6 + offset))
    //                         == 0
    //                 {
    //                     self.emit_move::<TYPE>(Move::Castle { is_short: true }, player, depth)
    //                 }
    //                 if !player_state.square_occupied(2 + offset)
    //                     && !player_state.square_occupied(3 + offset)
    //                     && !player_state.square_occupied(4 + offset)
    //                     && player_state.meta.can_castle_long
    //                     && self.opp_move_grid
    //                         | (1 << (4 + offset))
    //                         | (1 << (3 + offset))
    //                         | (1 << (2 + offset))
    //                         == 0
    //                 {
    //                     self.emit_move::<TYPE>(Move::Castle { is_short: false }, player, depth)
    //                 }
    //             }
    //         }
    //     }
    // }
    // // overrides move_buf
    // // todo: en-passant
    // fn get_moves_by_piece_type<const TYPE: bool>(
    //     &mut self,
    //     state: &GameState,
    //     piece: Piece,
    //     player: Player,
    //     depth: usize,
    // ) {
    //     for i in 0u8..64 {
    //         if state.get_state(player).pieces[piece as usize].0 & (1u64 << i) > 0 {
    //             self.get_move_by_piece::<TYPE>(state, piece, i, player, depth);
    //         }
    //     }
    // }

    // fn get_all_moves<const TYPE: bool>(&mut self, state: &GameState, player: Player, depth: usize) {
    //     self.get_moves_by_piece_type::<TYPE>(state, Piece::Pawn, player, depth);
    //     self.get_moves_by_piece_type::<TYPE>(state, Piece::Knight, player, depth);
    //     self.get_moves_by_piece_type::<TYPE>(state, Piece::Bishop, player, depth);
    //     self.get_moves_by_piece_type::<TYPE>(state, Piece::Rook, player, depth);
    //     self.get_moves_by_piece_type::<TYPE>(state, Piece::Queen, player, depth);
    //     self.get_moves_by_piece_type::<TYPE>(state, Piece::King, player, depth);
    // }

    // fn recurse(
    //     &mut self,
    //     state: &mut GameState,
    //     value: &mut i32,
    //     alpha: &mut i32,
    //     beta: &mut i32,
    //     rem_depth: usize,
    //     last_capture: Option<u8>
    // ) {

    // }
    // returns (score, initial piece pos, move piece pos)
    fn scoring_function(state: &GameState) -> i32{
        let cur = 
        eval::evaluate(state)
        // state.score()
        ;
        if state.player == Player::Black {-cur} else {cur}
    }

    fn calc(
        &mut self,
        state: &mut GameState,
        mut alpha: i32,
        mut beta: i32,
        depth: usize,
        last_move_pos: u8,
    ) -> i32 {
        self.nodes_explored += 1;
        if self.nodes_explored % 100000 == 0 {
            println!(
                "explored {} nodes, alpha: {}, beta: {}, cached_moves: {}, current_score: {}",
                self.nodes_explored, alpha, beta, self.cache_hits, eval::evaluate(state)
            );
        }

        // transposition table hit
        if let Some(MoveEntry{ value, .. }) = self.state_cache.get_entry(state.hash, depth as u8, state){
            self.cache_hits += 1;
            return value;
        }

        // let score = eval::evaluate(state);
        if depth == MAX_DEPTH_HARD {
            return Self::scoring_function(state);
        }

        let mut value =// if state.player == Player::White {
            - eval::WIN_THRESHOLD
        // } else {
        //     eval::WIN_THRESHOLD
        // }
        ;

        // if depth > MAX_DEPTH_CAPTURE {
        //     return (if value <= -WIN_THRESHOLD. || value >= WIN_THRESHOLD. {score as i32} else {value}, score as i32)
        // }

        self.move_buf.clear(depth);
        self.move_buf.get_all_moves::<true>(state, state.player, depth);

        // self.move_buf[depth].sort_by_key(|a| Reverse(a.get_cmp_key(last_move_pos, self.killer_table.get(depth))));
        
        let mut best_move = None;

        // self.move_buf.sort(depth, last_move_pos, self.killer_table.get(depth));

        while let Some(next_move) = self.move_buf.get_next_move(depth, last_move_pos, self.killer_table.get(depth)) {
        // while let Some(next_move) = self.move_buf.pop(depth){
            if let Move::Move {
                captured_piece: Some(Piece::King),
                ..
            } = next_move
            {
                let result = // if state.player == Player::White {
                    eval::WIN_THRESHOLD
                // } else {
                //     -eval::WIN_THRESHOLD
                // }
                ;
                if depth == 0 {
                    self.calculated_moves
                        .insert(ValueMovePair(result as i32, next_move));
                }
                return result;
            }
            // do the state transition

            if depth > MAX_DEPTH_SOFT {
                // after soft cap, non-captures are not permitted
                if let Move::Move {
                    captured_piece: Some(_),
                    new_pos,
                    ..
                } = next_move
                {
                    if depth > MAX_DEPTH_CAPTURE && new_pos != last_move_pos {
                        return if value <= -eval::WIN_THRESHOLD || value >= eval::WIN_THRESHOLD {
                            Self::scoring_function(state)
                        } else {
                            value
                        };
                    }
                } else {
                    return if value <= -eval::WIN_THRESHOLD || value >= eval::WIN_THRESHOLD {
                        Self::scoring_function(state)
                    } else {
                        value
                    };
                }
                // return if value <= -WIN_THRESHOLD. || value >= WIN_THRESHOLD. {score as i32} else {value}
            }

            if depth > MAX_DEPTH_CAPTURE {
                // return (if value <= -WIN_THRESHOLD. || value >= WIN_THRESHOLD. {score as i32} else {value}, score as i32)
                // return (0., 0.)
                // break;
            }

            // let last_state = state.clone();

            let meta = (state.states[0].meta, state.states[1].meta);
            state.apply_meta_hash(&self.zoborist_state);
            state.advance_state(next_move, &self.zoborist_state);
            state.apply_meta_hash(&self.zoborist_state);

            // update meta values
            let next_val = -self.calc(
                state,
                -beta,
                -alpha,
                depth + 1,
                if let Move::Move { new_pos, .. } = next_move {
                    new_pos
                } else {
                    64
                },
            );

            state.apply_meta_hash(&self.zoborist_state);
            state.revert_state(next_move, &self.zoborist_state);

            // revert metadata: easiest to do this way
            state.states[0].meta = meta.0;
            state.states[1].meta = meta.1;
            state.apply_meta_hash(&self.zoborist_state);

            // if *state != last_state{
            //     panic!("state no match");
            // }

            // if state.player == Player::White {
                if next_val > value{
                    value = next_val;
                    best_move = Some(next_move);
                }
                // value = i32::max(value, next_val);
                if value >= beta {
                    break;
                }
                alpha = i32::max(alpha, value);
            // } else {
            //     // value = i32::min(value, next_val);
            //     if next_val < value{
            //         value = next_val;
            //         best_move = Some(next_move);
            //     }
            //     if value <= alpha {
            //         break;
            //     }
            //     beta = i32::min(beta, value);
            // }

            if depth == 0 {
                self.calculated_moves
                    .insert(ValueMovePair(next_val as i32, next_move));
            }
        }

        if let Some(mov) = best_move{
            self.state_cache.insert_entry(MoveEntry{ hash: state.hash, mov, depth: depth as u8, value });
        }
        if let Some(Move::Move { captured_piece: None, .. }) = best_move{
            if let Some(best_move) = best_move{
                self.killer_table.insert(best_move, depth);
            }        
        }

        value
    }

    pub fn solve(&mut self, state: &mut GameState) -> i32 {
        state.slow_compute_hash(&self.zoborist_state);
        self.calc(state, - eval::WIN_THRESHOLD, eval::WIN_THRESHOLD, 0, 0)
    }

    pub fn get_moves(&mut self) {
        while let Some(ValueMovePair(value, m)) = self.calculated_moves.pop_last() {
            println!("{}: {}", value, m);
        }
    }
}
