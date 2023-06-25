use std::{cmp::Reverse, collections::BTreeSet};

use crate::{
    config::MOVE_TABLE_SIZE,
    eval,
    game_data::ZoboristState,
    killer_table::KillerTable,
    move_buffer::MoveBuffer,
    move_table::{MoveEntry, MoveTable},
    types::{Move, Piece, ValueMovePair},
    util, GameState, Player,
};

pub struct ChessEngine {
    // state: GameState,
    move_buf: MoveBuffer,
    killer_table: KillerTable,
    nodes_explored: u64,
    quiescence_nodes: u64,
    cache_hits: u64,
    calculated_moves: BTreeSet<ValueMovePair>,
    zoborist_state: ZoboristState,
    // state_cache: HashMap<HashType, (usize, i32), FxBuildHasher>
    state_cache: MoveTable<MOVE_TABLE_SIZE>,
    max_depth_encountered: usize,
    normal_depth: usize,
    quiescence_depth: usize,
}

// impl<const normal_depth: usize, const MAX_DEPTH_CAPTURE: usize, const quiescence_depth: usize>
//     Default for ChessEngine<normal_depth, MAX_DEPTH_CAPTURE, quiescence_depth>
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl ChessEngine
{
    pub fn new(normal_depth: usize, quiescence_depth: usize, zoborist_state_seed: u64) -> Self {
        if normal_depth > quiescence_depth {
            panic!("invalid depth parameters");
        }
        // const BUF: Vec<Move> = vec![];
        ChessEngine {
            move_buf: MoveBuffer::new(normal_depth + quiescence_depth),
            killer_table: KillerTable::new(normal_depth + quiescence_depth),
            nodes_explored: 0,
            quiescence_nodes: 0,
            cache_hits: 0,
            max_depth_encountered: 0,
            calculated_moves: BTreeSet::default(),
            zoborist_state: ZoboristState::new(zoborist_state_seed),
            // state_cache: HashMap::with_hasher(FxBuildHasher::default())
            state_cache: Default::default(),
            quiescence_depth,
            normal_depth
        }
    }

    // returns (score, initial piece pos, move piece pos)
    fn scoring_function(state: &GameState) -> i32 {
        let cur = eval::evaluate(state);
        if state.player == Player::Black {
            -cur
        } else {
            cur
        }
    }

    fn quiescence(
        &mut self,
        state: &mut GameState,
        mut alpha: i32,
        beta: i32,
        depth: usize,
        last_move_pos: u8,
    ) -> i32 {
        let stand_pat = Self::scoring_function(state);
        // self.max_depth_encountered = self.max_depth_encountered.min(depth);
        if depth == self.normal_depth + 1 {
            return stand_pat;
        }
        self.quiescence_nodes += 1;
        if (self.quiescence_nodes + self.nodes_explored) % 100000 == 0 {
            println!(
                "explored {} nodes, {}/{} normal/quiescence nodes, alpha: {}, beta: {}, cached_moves: {}, current_score: {}, max_depth_encountered: {}",
                self.nodes_explored + self.quiescence_nodes, self.nodes_explored, self.quiescence_nodes, alpha, beta, self.cache_hits, eval::evaluate(state), self.max_depth_encountered
            );
        }

        if stand_pat >= beta {
            return beta;
        }

        if let Some(MoveEntry { value, .. }) =
            self.state_cache.get_entry(state.hash, depth as u8, state)
        {
            self.cache_hits += 1;
            return value;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        self.move_buf.clear(depth);
        self.move_buf
            .get_all_moves::<true>(state, state.player, depth);

        if self.move_buf.is_zugzwang(depth) {
            return 0;
        }

        let mut best_move = None;

        while let Some(next_move) =
            self.move_buf
                .get_next_move(depth, last_move_pos, self.killer_table.get(depth))
        {
            if let Move::Move {
                captured_piece: Some(Piece::King),
                ..
            } = next_move
            {
                let result = eval::WIN_THRESHOLD;
                return result;
            }
            // do the state transition

            if let Move::Move {
                captured_piece: Some(_),
                new_pos,
                ..
            } = next_move
            {
                if new_pos != last_move_pos {
                    continue;
                }
                let meta = (state.states[0].meta, state.states[1].meta);
                state.apply_meta_hash(&self.zoborist_state);
                state.advance_state(next_move, &self.zoborist_state);
                state.apply_meta_hash(&self.zoborist_state);

                // update meta values
                let score = -self.quiescence(state, -beta, -alpha, depth - 1, last_move_pos);

                state.apply_meta_hash(&self.zoborist_state);
                state.revert_state(next_move, &self.zoborist_state);

                // revert metadata: easiest to do this way
                state.states[0].meta = meta.0;
                state.states[1].meta = meta.1;
                state.apply_meta_hash(&self.zoborist_state);

                if score >= beta {
                    return beta;
                }

                if score > alpha {
                    alpha = score;
                    best_move = Some(next_move);
                }
            }
        }
        if let Some(mov) = best_move {
            self.state_cache.insert_entry(MoveEntry {
                hash: state.hash,
                mov,
                depth: depth as u8,
                value: alpha,
            });
        }

        alpha
    }

    fn calc(
        &mut self,
        state: &mut GameState,
        mut alpha: i32,
        beta: i32,
        depth: usize,
        last_move_pos: u8,
    ) -> i32 {
        self.nodes_explored += 1;
        if (self.quiescence_nodes + self.nodes_explored) % 100000 == 0 {
            println!(
                "explored {} nodes, {}/{} normal/quiescence nodes, alpha: {}, beta: {}, cached_moves: {}, current_score: {}, max_depth_encountered: {}",
                self.nodes_explored + self.quiescence_nodes, self.nodes_explored, self.quiescence_nodes, alpha, beta, self.cache_hits, eval::evaluate(state), self.max_depth_encountered
            );
        }

        // transposition table hit
        if let Some(MoveEntry { value, .. }) =
            self.state_cache.get_entry(state.hash, depth as u8, state)
        {
            self.cache_hits += 1;
            return value;
        }

        let mut value = -eval::WIN_THRESHOLD;

        self.move_buf.clear(depth);
        self.move_buf
            .get_all_moves::<true>(state, state.player, depth);

        if self.move_buf.is_zugzwang(depth) {
            return 0;
        }

        // self.move_buf[depth].sort_by_key(|a| Reverse(a.get_cmp_key(last_move_pos, self.killer_table.get(depth))));

        let mut best_move = None;

        // self.move_buf.sort(depth, last_move_pos, self.killer_table.get(depth));

        while let Some(next_move) =
            self.move_buf
                .get_next_move(depth, last_move_pos, self.killer_table.get(depth))
        {
            // while let Some(next_move) = self.move_buf.pop(depth){
            if let Move::Move {
                captured_piece: Some(Piece::King),
                ..
            } = next_move
            {
                let result = eval::WIN_THRESHOLD;
                if depth == 0 {
                    self.calculated_moves
                        .insert(ValueMovePair(result as i32, next_move));
                }
                return result;
            }
            // do the state transition
            // let last_state = state.clone();

            let meta = (state.states[0].meta, state.states[1].meta);
            state.apply_meta_hash(&self.zoborist_state);
            state.advance_state(next_move, &self.zoborist_state);
            state.apply_meta_hash(&self.zoborist_state);

            let last_move_pos = if let Move::Move { new_pos, .. } = next_move {
                new_pos
            } else if let Move::PawnPromote { new_pos, .. } = next_move {
                new_pos
            } else {
                64
            };

            // update meta values
            let next_val = if depth > self.quiescence_depth {
                -self.calc(state, -beta, -alpha, depth - 1, last_move_pos)
            } else if next_move.is_capture() {
                -self.quiescence(state, -beta, -alpha, depth - 1, last_move_pos)
                // Self::scoring_function(state)                
            }
            else {
                -Self::scoring_function(state)
            };

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
            if next_val > value {
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

            if depth == self.normal_depth + self.quiescence_depth {
                self.calculated_moves
                    .insert(ValueMovePair(next_val as i32, next_move));
            }
        }

        if let Some(mov) = best_move {
            self.state_cache.insert_entry(MoveEntry {
                hash: state.hash,
                mov,
                depth: depth as u8,
                value,
            });
        }
        if let Some(Move::Move {
            captured_piece: None,
            ..
        }) = best_move
        {
            if let Some(best_move) = best_move {
                self.killer_table.insert(best_move, depth);
            }
        }

        value
    }

    pub fn solve(&mut self, state: &mut GameState) -> i32 {
        state.slow_compute_hash(&self.zoborist_state);
        self.calc(state, -eval::WIN_THRESHOLD, eval::WIN_THRESHOLD, self.normal_depth + self.quiescence_depth, 0)
    }

    pub fn get_result(&mut self) {
        println!("nodes: {} ({}/{}), cached_nodes: {}", self.nodes_explored + self.quiescence_nodes, self.nodes_explored, self.quiescence_nodes, self.cache_hits);
        while let Some(ValueMovePair(value, m)) = self.calculated_moves.pop_last() {
            println!("{}: {}", value, m);
        }
    }
    
    pub fn perft(&mut self, state: &mut GameState, depth: usize) -> i64 {       
        self.move_buf.clear(depth);
        self.move_buf
            .get_all_moves::<true>(state, state.player, depth);
        
        if self.move_buf.is_zugzwang(depth) {
            return 0;
        }
        
        if !self.move_buf.is_legal(depth) {
            // println!("hi2");
            return -1;
        }
        
        if depth == 0 {
            return 1;
        }
        
        let mut cnt = 0;

        // println!("hi");

        let mut has_legal_child = false;

        while let Some(next_move) =
            self.move_buf
                .pop(depth)
        {
            let meta = (state.states[0].meta, state.states[1].meta);
            state.advance_state(next_move, &self.zoborist_state);

            let res = self.perft(state, depth - 1);
            if res >= 0 {
                cnt += res;
                has_legal_child = true;
            }

            state.revert_state(next_move, &self.zoborist_state);
            state.states[0].meta = meta.0;
            state.states[1].meta = meta.1;
        }

        // if no legal children, it is a checkmate
        if !has_legal_child{
            cnt = 1;
        }

        cnt
    }
}
