use std::{collections::BTreeSet, fmt::Display};

use crate::{
    config::{MOVE_TABLE_SIZE, NULL_MOVES_PER_BRANCH, NULL_MOVE_DEPTH_REDUCTION},
    eval,
    game_data::ZoboristState,
    move_buffer::MoveBuffer,
    move_orderer::MoveOrderer,
    move_table::{MoveEntry, MoveTable},
    types::{Move, Piece, ValueMovePair},
    GameState, Player,
};

struct EngineStatistics {
    nodes_explored: u64,
    quiescence_nodes: u64,
    cache_hits: u64,
    max_depth_encountered: usize,
    null_move_fail_highs: usize,
}

impl Default for EngineStatistics {
    fn default() -> Self {
        Self {
            nodes_explored: Default::default(),
            quiescence_nodes: Default::default(),
            cache_hits: Default::default(),
            max_depth_encountered: Default::default(),
            null_move_fail_highs: Default::default(),
        }
    }
}

impl Display for EngineStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = format!(
            "nodes: {} ({}/{}), cached_nodes: {}, null move fail highs: {}",
            self.nodes_explored + self.quiescence_nodes,
            self.nodes_explored,
            self.quiescence_nodes,
            self.cache_hits,
            self.null_move_fail_highs
        );
        f.write_str(&res)
    }
}

pub struct ChessEngine {
    // state: GameState,
    // move_buf: MoveBuffer,
    move_orderer: MoveOrderer,
    calculated_moves: BTreeSet<ValueMovePair>,
    zoborist_state: ZoboristState,
    // state_cache: HashMap<HashType, (usize, i32), FxBuildHasher>
    state_cache: MoveTable<MOVE_TABLE_SIZE>,
    normal_depth: usize,
    quiescence_depth: usize,
    stats: EngineStatistics,
}

// impl<const normal_depth: usize, const MAX_DEPTH_CAPTURE: usize, const quiescence_depth: usize>
//     Default for ChessEngine<normal_depth, MAX_DEPTH_CAPTURE, quiescence_depth>
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl ChessEngine {
    pub fn new(normal_depth: usize, quiescence_depth: usize, zoborist_state_seed: u64) -> Self {
        if normal_depth > quiescence_depth {
            panic!("invalid depth parameters");
        }
        // const BUF: Vec<Move> = vec![];
        ChessEngine {
            // move_buf: MoveBuffer::new(normal_depth + quiescence_depth),
            move_orderer: MoveOrderer::new(normal_depth + quiescence_depth),
            calculated_moves: BTreeSet::default(),
            zoborist_state: ZoboristState::new(zoborist_state_seed),
            // state_cache: HashMap::with_hasher(FxBuildHasher::default())
            state_cache: Default::default(),
            quiescence_depth,
            normal_depth,
            stats: Default::default(),
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
        self.stats.quiescence_nodes += 1;
        if (self.stats.quiescence_nodes + self.stats.nodes_explored) % 100000 == 0 {
            println!(
                "explored {} nodes, {}/{} normal/quiescence nodes, alpha: {}, beta: {}, cached_moves: {}, current_score: {}, max_depth_encountered: {}",
                self.stats.nodes_explored + self.stats.quiescence_nodes, self.stats.nodes_explored, self.stats.quiescence_nodes, alpha, beta, self.stats.cache_hits, eval::evaluate(state), self.stats.max_depth_encountered
            );
        }

        if stand_pat >= beta {
            return beta;
        }

        if let Some(MoveEntry { value, .. }) =
            self.state_cache
                .get_entry(state.hash, self.quiescence_depth as u8, state)
        {
            self.stats.cache_hits += 1;
            return value;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let mut move_buf = MoveBuffer::default();

        // self.move_buf.clear(depth);
        move_buf.get_all_moves::<true>(state, state.player);

        if move_buf.is_zugzwang() {
            return 0;
        }

        let mut best_move = None;

        while let Some(next_move) =
            move_buf.get_next_move(last_move_pos, depth, state.player, &mut self.move_orderer)
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
                depth: self.quiescence_depth as u8,
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
        allow_null_move: bool,
        null_move_count: u8,
        is_root: bool,
    ) -> i32 {
        if depth <= self.quiescence_depth {
            return self.quiescence(state, alpha, beta, self.quiescence_depth, last_move_pos);
        }

        self.stats.nodes_explored += 1;
        if (self.stats.quiescence_nodes + self.stats.nodes_explored) % 100000 == 0 {
            println!(
                "explored {} nodes, {}/{} normal/quiescence nodes, alpha: {}, beta: {}, cached_moves: {}, current_score: {}, max_depth_encountered: {}",
                self.stats.nodes_explored + self.stats.quiescence_nodes, self.stats.nodes_explored, self.stats.quiescence_nodes, alpha, beta, self.stats.cache_hits, eval::evaluate(state), self.stats.max_depth_encountered
            );
        }

        // transposition table hit
        if let Some(MoveEntry { value, .. }) =
            self.state_cache.get_entry(state.hash, depth as u8, state)
        {
            self.stats.cache_hits += 1;
            return value;
        }

        if allow_null_move
            && null_move_count != NULL_MOVES_PER_BRANCH
            && depth > self.quiescence_depth + 1
        {
            state.change_player(&self.zoborist_state);

            let next_val = -self.calc(
                state,
                -beta,
                -beta + 1,
                depth - 1 - NULL_MOVE_DEPTH_REDUCTION,
                last_move_pos,
                false,
                null_move_count + 1,
                false,
            );

            state.change_player(&self.zoborist_state);

            if next_val >= beta {
                self.stats.null_move_fail_highs += 1;
                return next_val;
            }
        }

        let mut value = -eval::WIN_THRESHOLD;

        let mut move_buf = MoveBuffer::default();
        // move_buf.clear(depth);
        move_buf.get_all_moves::<true>(state, state.player);

        if move_buf.is_zugzwang() {
            return 0;
        }

        // self.move_buf[depth].sort_by_key(|a| Reverse(a.get_cmp_key(last_move_pos, self.killer_table.get(depth))));

        let mut best_move = None;

        // self.move_buf.sort(depth, last_move_pos, self.killer_table.get(depth));

        while let Some(next_move) =
            move_buf.get_next_move(last_move_pos, depth, state.player, &mut self.move_orderer)
        {
            // while let Some(next_move) = self.move_buf.pop(depth){
            if is_root {
                println!("{}", next_move);
            }
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
            // let next_val = if depth > self.quiescence_depth {
            //     -self.calc(state, -beta, -alpha, depth - 1, last_move_pos, true)
            // } else if next_move.is_capture() {
            //     -self.quiescence(state, -beta, -alpha, depth - 1, last_move_pos)
            //     // Self::scoring_function(state)
            // }
            // else {
            //     -Self::scoring_function(state)
            // };

            let next_val = -self.calc(
                state,
                -beta,
                -alpha,
                depth - 1,
                last_move_pos,
                true,
                null_move_count,
                false,
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
            if next_val > value {
                value = next_val;
                best_move = Some(next_move);
            }
            // value = i32::max(value, next_val);
            if is_root {
                self.calculated_moves
                    .insert(ValueMovePair(next_val as i32, next_move));
            }

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
        }

        if let Some(mov) = best_move {
            self.state_cache.insert_entry(MoveEntry {
                hash: state.hash,
                mov,
                depth: depth as u8,
                value,
            });
            self.move_orderer.update_history(
                mov,
                state.player,
                depth - self.quiescence_depth + 1,
            );
        }
        if let Some(Move::Move {
            captured_piece: None,
            ..
        }) = best_move
        {
            if let Some(best_move) = best_move {
                self.move_orderer.insert_killer_move(best_move, depth);
            }
        }

        value
    }

    pub fn solve(&mut self, state: &GameState, depth: usize) -> i32 {
        assert!(depth <= self.normal_depth);
        let mut state = state.clone();
        state.setup(&self.zoborist_state);
        self.calculated_moves.clear();
        let result = self.calc(
            &mut state,
            -eval::WIN_THRESHOLD,
            eval::WIN_THRESHOLD,
            depth + self.quiescence_depth,
            0,
            true,
            0,
            true,
        );
        if state.player == Player::Black {
            let mut new_moves: BTreeSet<ValueMovePair> = Default::default();
            for item in self.calculated_moves.iter() {
                let new_item = ValueMovePair(item.0 * -1, item.1);
                new_moves.insert(new_item);
            }
            self.calculated_moves = new_moves;
        }
        result
    }

    pub fn get_result(&mut self) {
        println!("{}", self.stats);
        while let Some(ValueMovePair(value, m)) = self.calculated_moves.pop_last() {
            println!("{}: {}", value, m);
        }
    }

    pub fn perft(&mut self, state: &mut GameState, depth: usize) -> i64 {
        let mut move_buf = MoveBuffer::default();
        // move_buf.clear(depth);
        move_buf.get_all_moves::<true>(state, state.player);

        if move_buf.is_zugzwang() {
            return 0;
        }

        if !move_buf.is_legal() {
            // println!("hi2");
            return -1;
        }

        if depth == 0 {
            return 1;
        }

        let mut cnt = 0;

        // println!("hi");

        let mut has_legal_child = false;

        while let Some(next_move) = move_buf.pop() {
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
        if !has_legal_child {
            cnt = 1;
        }

        cnt
    }
}
