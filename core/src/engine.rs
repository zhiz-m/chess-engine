use std::fmt::Display;

use crate::{
    config::{MOVE_TABLE_SIZE, NULL_MOVES_PER_BRANCH, NULL_MOVE_DEPTH_REDUCTION, HashType},
    eval,
    grid::Grid,
    markers::{BlackMarker, WhiteMarker},
    move_buffer::MoveBuffer,
    move_orderer::MoveOrderer,
    move_table::{MoveEntry, MoveTable},
    player::Player,
    types::{Move, ValueMovePair},
    zoborist_state::ZoboristState,
    GameState, game_data::Metadata, evaluate, Piece, square_type::{SquareType, CompressedSquareType},
};

struct EngineStatistics {
    nodes_explored: u64,
    terminal_nodes: u64,
    quiescence_nodes: u64,
    cache_direct_cutoff_hits: u64,
    cache_hits: u64,
    max_depth_encountered: usize,
    null_move_fail_highs: usize,
    cutoffs: usize,
    cutoffs_perfect_move_orderings: usize,
}

impl Default for EngineStatistics {
    fn default() -> Self {
        Self {
            nodes_explored: 0,
            terminal_nodes: 0,
            quiescence_nodes: Default::default(),
            cache_direct_cutoff_hits: Default::default(),
            cache_hits: Default::default(),
            max_depth_encountered: 999,
            null_move_fail_highs: Default::default(),
            cutoffs: 0,
            cutoffs_perfect_move_orderings: 0,
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
    move_bufs: Vec<MoveBuffer>,
    move_orderer: MoveOrderer,
    calculated_moves: Vec<ValueMovePair>,
    pub zoborist_state: ZoboristState,
    // state_cache: HashMap<HashType, (usize, i32), FxBuildHasher>
    state_cache: MoveTable<MOVE_TABLE_SIZE>,
    normal_depth: usize,
    quiescence_depth: usize,
    visited_nodes: Vec<HashType>,
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
            move_bufs: vec![MoveBuffer::default(); normal_depth + quiescence_depth],
            move_orderer: MoveOrderer::new(normal_depth + quiescence_depth),
            calculated_moves: Default::default(),
            zoborist_state: ZoboristState::new(zoborist_state_seed),
            // state_cache: HashMap::with_hasher(FxBuildHasher::default())
            state_cache: Default::default(),
            quiescence_depth,
            normal_depth,
            visited_nodes: Vec::with_capacity(normal_depth*2),
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

    pub fn print_debug(&self) {
        println!("{} nodes, {} terminal nodes", self.stats.nodes_explored + self.stats.quiescence_nodes, self.stats.terminal_nodes, )
    }

    fn try_print_debug(&self, alpha: i32, beta: i32, state: &GameState) {
        if (self.stats.quiescence_nodes + self.stats.nodes_explored) % 100000 == 0 {
            println!(
                "explored {} nodes, {} terminal nodes, {} branching factor, {}/{} normal/quiescence nodes, {} cutoffs, {} perfect cutoffs, alpha: {}, beta: {}, direct cache hits: {}, cache hits: {}, current_score: {}, max_depth_encountered: {}",
                self.stats.nodes_explored + self.stats.quiescence_nodes, self.stats.terminal_nodes, 
                (self.stats.nodes_explored as f64 + self.stats.quiescence_nodes as f64) / (self.stats.nodes_explored as f64+ self.stats.quiescence_nodes as f64 - self.stats.terminal_nodes as f64),
                 self.stats.nodes_explored, self.stats.quiescence_nodes, self.stats.cutoffs, self.stats.cutoffs_perfect_move_orderings, alpha, beta, self.stats.cache_direct_cutoff_hits, self.stats.cache_hits, eval::evaluate(state), self.stats.max_depth_encountered
            );
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
        self.stats.quiescence_nodes += 1;
        self.stats.terminal_nodes += 1;
        let stand_pat = Self::scoring_function(state);
        // self.stats.max_depth_encountered = self.stats.max_depth_encountered.min(depth);
        if depth == self.normal_depth + 1 {
            self.stats.terminal_nodes -= 1;
            return stand_pat;
        }
        self.try_print_debug(alpha, beta, state);

        if stand_pat >= beta {
            self.stats.terminal_nodes -= 1;
            return beta;
        }
        // let move_entry =
        //     self.state_cache
        //         .get_entry_for_ordering(state.hash, self.quiescence_depth as u8, state);

        // if let Some(MoveEntry { value, .. }) = move_entry {
        //     self.stats.cache_direct_cutoff_hits += 1;
        //     if value >= beta {
        //         self.stats.cutoffs += 1;
        //         self.stats.cutoffs_perfect_move_orderings += 1;
        //         return value;
        //     }
        // }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        // let move_buf = &mut self.move_bufs[depth];
        self.move_bufs[depth].clear();
        self.move_bufs[depth].get_all_captures(state);
        self.move_bufs[depth].compute_see(state);

        // if self.move_bufs[depth].is_stalemate() {
        //     return 0;
        // }

        let mut best_move = None;
        let mut first_move_explored = true;

        while let Some(next_move) = self.move_bufs[depth].get_quiescence_move(
            state,
            last_move_pos,
            depth,
            state.player,
            None,
            &mut self.move_orderer,
        ) {
            match next_move {
                Move::Move {
                    pieces,
                    new_pos,
                    ..
                }
                | Move::PawnPromote {
                    new_pos,
                    pieces,
                    ..
                } => {
                    let (_, captured_piece) = pieces.to_square_types();
                    if captured_piece.is_king() {
                        let result = eval::SCORE_AFTER_KING_CAPTURED;
                        return result;
                    }
                    if captured_piece.is_empty() {
                        panic!("unexpected, captured piece should not be empty");
                    }
                    if captured_piece.is_empty() || new_pos != last_move_pos {
                        // println!("hi");
                        // first_move_explored = false;
                        // continue;
                    }
                    let metadata = state.metadata;
                    state.apply_meta_hash(&self.zoborist_state);
                    state.advance_state(next_move, &self.zoborist_state);
                    state.apply_meta_hash(&self.zoborist_state);
                    
                    // futility pruning
                    // if evaluate(state) + 2 * eval::PAWN_VALUE < alpha {
                    //     state.apply_meta_hash(&self.zoborist_state);
                    //     state.revert_state(next_move, &self.zoborist_state);

                    //     // revert metadata: easiest to do this way
                    //     state.metadata = metadata;
                    //     state.apply_meta_hash(&self.zoborist_state);
                    //     continue;
                    // }

                    // update meta values
                    let score = -self.quiescence(state, -beta, -alpha, depth - 1, last_move_pos);

                    state.apply_meta_hash(&self.zoborist_state);
                    state.revert_state(next_move, &self.zoborist_state);

                    // revert metadata: easiest to do this way
                    state.metadata = metadata;
                    state.apply_meta_hash(&self.zoborist_state);

                    if score > alpha {
                        alpha = score;
                        best_move = Some(next_move);
                    }
                    if score >= beta {
                        self.stats.cutoffs += 1;
                        if first_move_explored {
                            self.stats.cutoffs_perfect_move_orderings += 1;
                        }
                        // return beta;
                        break;
                    }

                }
                _ => {}
            }
            first_move_explored = false;
        }
        // if let Some(mov) = best_move {
        //     self.state_cache.insert_entry(MoveEntry {
        //         hash: state.hash,
        //         mov,
        //         depth: self.quiescence_depth as u8,
        //         value: alpha,
        //     });
        // }

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
        self.stats.nodes_explored += 1;
        self.stats.terminal_nodes += 1;
        if self.visited_nodes.contains(&state.hash){
            // repetition.
            self.stats.terminal_nodes -= 1;
            return 0;
        }
        self.visited_nodes.push(state.hash);

        // transposition table hit
        if let Some(MoveEntry { value, .. }) = self.state_cache.get_entry_for_direct_cutoff(
            state.hash,
            self.quiescence_depth as u8 + depth as u8,
            state,
        ) {
            self.stats.cache_hits += 1;
            if value >= beta {
                self.stats.cutoffs += 1;
                self.stats.cutoffs_perfect_move_orderings += 1;
                self.visited_nodes.pop().unwrap();
                self.stats.terminal_nodes -= 1;
                return value;
            }
            // return value;
        }
        // self.stats.max_depth_encountered = self.stats.max_depth_encountered.min(depth);
        if depth <= self.quiescence_depth {
            self.visited_nodes.pop().unwrap();
            self.stats.nodes_explored -= 1;
            self.stats.terminal_nodes -= 1;
            return self.quiescence(state, alpha, beta, self.quiescence_depth, last_move_pos);
        }

        self.try_print_debug(alpha, beta, state);


        let move_entry = self.state_cache.get_entry_for_ordering(
            state.hash,
            self.quiescence_depth as u8 + depth as u8,
            state,
        );

        if allow_null_move
            && null_move_count != NULL_MOVES_PER_BRANCH
            && depth > self.quiescence_depth + 1
        {
            let metadata = state.metadata;
            state.apply_meta_hash(&self.zoborist_state);
            state.change_player(&self.zoborist_state);
            state.metadata.set_en_passant_column(Metadata::NO_EN_PASSANT);
            state.apply_meta_hash(&self.zoborist_state);

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

            state.apply_meta_hash(&self.zoborist_state);
            state.metadata = metadata;
            state.apply_meta_hash(&self.zoborist_state);

            if next_val >= beta {
                self.stats.null_move_fail_highs += 1;
                self.visited_nodes.pop().unwrap();
                return next_val;
            }
        }

        let mut value = -eval::SCORE_MAX;

        // let mut move_buf = MoveBuffer::default();
        // move_buf.clear(depth);
        self.move_bufs[depth].clear();
        self.move_bufs[depth].get_all_moves(state);

        if self.move_bufs[depth].is_stalemate() {
            self.visited_nodes.pop().unwrap();
            return 0;
        }

        // self.move_buf[depth].sort_by_key(|a| Reverse(a.get_cmp_key(last_move_pos, self.killer_table.get(depth))));

        let mut best_move = None;

        // self.move_buf.sort(depth, last_move_pos, self.killer_table.get(depth));
        let mut first_move_explored = false;
        let mut has_cutoff = false;

        macro_rules! loop_inner {
            ($next_move:ident) => {{
                // while let Some(next_move) = self.move_buf.pop(depth){
                if is_root {
                    println!("{}", $next_move);
                }

                let try_store_move = |engine: &mut Self, next_val| {
                    if is_root {
                        if engine
                            .calculated_moves
                            .iter()
                            .find(|x| x.0 == next_val)
                            .is_none()
                        {
                            engine
                                .calculated_moves
                                .push(ValueMovePair(next_val, $next_move));
                            // println!("try store move {} {}", next_val, $next_move);
                        }
                    }
                };
                match $next_move {
                    Move::Move { pieces, .. }
                    | Move::PawnPromote { pieces, .. } => {
                        let (_, captured_piece) = pieces.to_square_types();
                        if captured_piece.is_king() {
                            let result = eval::SCORE_AFTER_KING_CAPTURED;
                            try_store_move(self, result);
                            self.visited_nodes.pop().unwrap();
                            return result;
                        }
                    }
                    _ => {}
                }
                // do the state transition
                // let last_state = state.clone();

                let metadata = state.metadata;
                state.apply_meta_hash(&self.zoborist_state);
                state.advance_state($next_move, &self.zoborist_state);
                state.apply_meta_hash(&self.zoborist_state);

                let last_move_pos = if let Move::Move { new_pos, .. } = $next_move {
                    new_pos
                } else if let Move::PawnPromote { new_pos, .. } = $next_move {
                    new_pos
                } else {
                    64
                };

                // update meta values
                // let next_val = if depth > self.quiescence_depth {
                //     -self.calc(state, -beta, -alpha, depth - 1, last_move_pos, true)
                // } else if $next_move.is_capture() {
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
                state.revert_state($next_move, &self.zoborist_state);

                // revert metadata: easiest to do this way
                state.metadata = metadata;
                state.apply_meta_hash(&self.zoborist_state);

                // if *state != last_state{
                //     panic!("state no match");
                // }

                // if state.player == Player::White {
                if next_val > value {
                    value = next_val;
                    best_move = Some($next_move);
                }
                // value = i32::max(value, next_val);

                try_store_move(self, next_val);
                if value >= beta{
                    self.stats.cutoffs += 1;
                    if first_move_explored {
                        self.stats.cutoffs_perfect_move_orderings += 1;
                    }
                    has_cutoff = true;
                    break;
                }
                alpha = i32::max(alpha, value);
                // } else {
                //     // value = i32::min(value, next_val);
                //     if next_val < value{
                //         value = next_val;
                //         best_move = Some($next_move);
                //     }
                //     if value <= alpha {
                //         break;
                //     }
                //     beta = i32::min(beta, value);
                // }
                first_move_explored = false;
            }};
        }

        while let Some(next_move) = self.move_bufs[depth].get_next_move_capture_king() {
            loop_inner!(next_move);
        }

        if !has_cutoff {
            while let Some(next_move) =
                self.move_bufs[depth].get_next_move_exact_match(move_entry.map(|x| x.mov))
            {
                self.stats.cache_hits += 1;
                loop_inner!(next_move);
            }
        }

        if !has_cutoff {
            self.move_bufs[depth].compute_see(state);
            while let Some(next_move) = self.move_bufs[depth].get_next_move_positive_equal_capture(
                state,
                last_move_pos,
                depth,
                state.player,
                move_entry.map(|x| x.mov),
                &mut self.move_orderer,
            ) {
                loop_inner!(next_move);
            }
        }

        if !has_cutoff {
            while let Some(next_move) =
                self.move_bufs[depth].get_next_move_killer(depth, &mut self.move_orderer)
            {
                loop_inner!(next_move);
            }
        }

        if !has_cutoff {
            while let Some(next_move) = self.move_bufs[depth].get_next_move(
                state,
                last_move_pos,
                depth,
                state.player,
                move_entry.map(|x| x.mov),
                &mut self.move_orderer,
            ) {
                loop_inner!(next_move);
            }
        }

        // no legal moves
        if best_move == None || value < -eval::SCORE_AFTER_KING_CAPTURED_CUTOFF {
            // determine if currently in check
            let (attacked_grid, king_pos) = match state.player {
                Player::White => (
                    MoveBuffer::get_attacked_grid::<BlackMarker>(state),
                    state.piece_grid.get_king_pos::<WhiteMarker>(),
                ),
                Player::Black => (
                    MoveBuffer::get_attacked_grid::<WhiteMarker>(state),
                    state.piece_grid.get_king_pos::<BlackMarker>(),
                ),
            };
            // self.move_bufs[depth].clear();
            // self.move_bufs[depth].get_squares_under_attack(state);
            // for pos in 0u8..64 {
            //     if state.get_state(state.player).pieces[Piece::King as usize].0 & (1u64 << pos) > 0
            //     {
            //         if self.move_bufs[depth].is_square_under_attack(pos) {
            //             // checkmate
            //             return -eval::WIN_THRESHOLD - (depth as i32);
            //         }
            //     }
            // }

            // checkmate
            if attacked_grid & king_pos != Grid::EMPTY {
                self.visited_nodes.pop().unwrap();
                return -eval::WIN_THRESHOLD - (depth as i32);
            }

            // stalemate
            self.visited_nodes.pop().unwrap();
            return 0;
        }

        if let Some(mov) = best_move {
            self.state_cache.insert_entry(MoveEntry {
                hash: state.hash,
                mov,
                depth: depth as u8,
                value,
            });
            self.move_orderer
                .update_history(mov, state.player, depth - self.quiescence_depth + 1);
        }
        if let Some(best_move @ Move::Move { pieces, .. }) = best_move {
            let (_, captured_piece) = pieces.to_square_types();
            if captured_piece.is_empty() {
                self.move_orderer.insert_killer_move(best_move, depth);
            }
        }

        self.visited_nodes.pop().unwrap();
        value
    }

    pub fn solve(&mut self, state: &GameState, depth: usize) -> i32 {
        // the engine will never visit the same state more than once. hence, to avoid threefold repetition,
        // we only care about previous moves that appear more than once. 
        println!("static eval: {}", eval::evaluate(state));
        let mut new_visited_nodes = vec![];
        for item in self.visited_nodes.iter(){
            if self.visited_nodes.iter().filter(|x|**x == *item).count() > 1{
                new_visited_nodes.push(*item);
            }
        }
        self.visited_nodes = new_visited_nodes;
        assert!(depth <= self.normal_depth);
        let mut state = state.clone();
        // state.setup(&self.zoborist_state);
        self.calculated_moves.clear();
        let mut result = self.calc(
            &mut state,
            -eval::SCORE_MAX,
            eval::SCORE_MAX,
            depth + self.quiescence_depth,
            0,
            true,
            0,
            true,
        );
        if state.player == Player::Black {
            /*let mut new_moves: BTreeSet<ValueMovePair> = Default::default();
            for item in self.calculated_moves.iter() {
                let new_item = ValueMovePair(item.0 * -1, item.1);
                new_moves.insert(new_item);
            }
            self.calculated_moves = new_moves;
            */
            for ValueMovePair(val, _) in self.calculated_moves.iter_mut() {
                *val *= -1;
            }
            self.calculated_moves.sort();
            result *= -1;
        }
        result
    }

    pub fn get_best_calculated_move(&mut self, player: Player) -> Option<Move> {
        self.calculated_moves.sort();
        match player {
            Player::White => self.calculated_moves.last().map(|pair| pair.1),
            Player::Black => self.calculated_moves.first().map(|pair| pair.1),
        }
    }

    pub fn get_result(&mut self) {
        println!("{}", self.stats);
        self.calculated_moves.sort();
        while let Some(ValueMovePair(value, m)) = self.calculated_moves.pop() {
            println!("{}: {}", value, m);
        }
    }

    pub fn perft(&mut self, state: &mut GameState, depth: usize) -> i64 {
        // determine if cn capture the opposing king
        let (attacked_grid, king_pos) = match state.player {
            Player::Black => (
                MoveBuffer::get_attacked_grid::<BlackMarker>(state),
                state.piece_grid.get_king_pos::<WhiteMarker>(),
            ),
            Player::White => (
                MoveBuffer::get_attacked_grid::<WhiteMarker>(state),
                state.piece_grid.get_king_pos::<BlackMarker>(),
            ),
        };

        // checkmate
        if attacked_grid & king_pos != Grid::EMPTY {
            return 0;
        }

        if depth == 0 {
            return 1;
        }

        let move_entry = self
            .state_cache
            .get_entry_perft(state.hash, depth.try_into().unwrap());

        if let Some(MoveEntry { value, .. }) = move_entry {
            self.stats.cache_hits += 1;
            return value as i64;
        }

        // let mut move_buf = MoveBuffer::default();
        // move_buf.clear(depth);
        self.move_bufs[depth].clear();
        self.move_bufs[depth].get_all_moves(state);

        // if !self.move_bufs[depth].is_legal() {
        //     // println!("hi2");
        //     return -1;
        // }

        // if depth == 0 {
        //     return 1;
        // }

        let mut cnt: i64 = 0;

        // println!("hi");

        // let map = HashMap::default();

        while let Some(next_move) = self.move_bufs[depth].pop() {
            // if depth == 2 {
            //     map.get(k)
            //     println!("move: {}", next_move);
            // }
            let metadata = state.metadata;
            state.apply_meta_hash(&self.zoborist_state);
            state.advance_state(next_move, &self.zoborist_state);
            state.apply_meta_hash(&self.zoborist_state);

            let res = self.perft(state, depth - 1);

            cnt += res;

            state.apply_meta_hash(&self.zoborist_state);
            state.revert_state(next_move, &self.zoborist_state);
            state.metadata = metadata;
            state.apply_meta_hash(&self.zoborist_state);
        }

        // if no legal children, it is a checkmate
        cnt = cnt.max(1);

        self.state_cache.insert_entry(MoveEntry {
            hash: state.hash,
            mov: Move::Castle { is_short: false },
            depth: depth.try_into().unwrap(),
            value: cnt as i32,
        });

        cnt
    }
    
    pub fn clear_move_history_threefold_repetition(&mut self){
        self.visited_nodes.clear();
    }

    pub fn make_move_raw_parts(
        &mut self,
        game_state: &mut GameState,
        prev_pos: u8,
        new_pos: u8,
        promoted_to_piece: Option<Piece>,
    ) -> Result<(), &'static str> {
        let piece = game_state.piece_grid.get_square_type(prev_pos);
        let captured_piece = game_state.piece_grid.get_square_type(new_pos);
        let next_move = if let Some(promoted_to_piece) = promoted_to_piece {
            let promoted_to_piece = SquareType::create_for_parsing(promoted_to_piece, game_state.player);
            let pieces = CompressedSquareType::from_square_types(promoted_to_piece, captured_piece);
            Move::PawnPromote {
                prev_pos,
                new_pos,
                pieces
            }
        } else if piece.is_king() && i32::abs(prev_pos as i32 - new_pos as i32) == 2 {
            Move::Castle {
                is_short: new_pos < prev_pos,
            }
        } else if piece.is_pawn()
            && (new_pos & 0b111 != prev_pos & 0b111)
            && captured_piece.is_empty()
        {
            Move::EnPassant {
                prev_column: prev_pos & 0b111,
                new_column: new_pos & 0b111,
            }
        } else {
            let pieces = CompressedSquareType::from_square_types(piece, captured_piece);
            Move::Move {
                prev_pos,
                new_pos,
                pieces
            }
        };

        self.visited_nodes.push(game_state.hash);
        game_state.advance_state(next_move, &self.zoborist_state);
        Ok(())
    }

    pub fn lift_killer_moves(&mut self, depth_to_lift: usize){
        self.move_orderer.lift_killer_moves(depth_to_lift)
    }
}
