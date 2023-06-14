use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use rand::Rng;
use std::{cmp::Reverse, collections::BTreeSet, fmt::Display};
use crate::{util, config::{PIECE_SCORES, NUM_PIECES, HASH_TYPE}, game_data::{ZoboristState, GameState}};
use std::hash::Hash;


#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    pub fn opp(self) -> Player {
        if self == Player::White {
            Player::Black
        } else {
            Player::White
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    fn value(self) -> i32 {
        // [1, 3, 3, 5, 9, 1_000_000_000]
        PIECE_SCORES[self as usize]
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Piece::Pawn => "pawn",
            Piece::Knight => "knight",
            Piece::Bishop => "bishop",
            Piece::Rook => "rook",
            Piece::Queen => "queen",
            Piece::King => "king",
        })
    }
}

impl From<usize> for Piece {
    fn from(value: usize) -> Self {
        match value {
            0 => Piece::Pawn,
            1 => Piece::Knight,
            2 => Piece::Bishop,
            3 => Piece::Rook,
            4 => Piece::Queen,
            5 => Piece::King,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid(u64);

impl<'a, T> From<T> for Grid
where
    T: IntoIterator<Item = &'a str>,
{
    fn from(value: T) -> Self {
        let mut grid = 0;
        for pos in value {
            grid |= 1u64 << util::canonical_to_pos(pos);
        }
        Grid(grid)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Move {
        prev_pos: u8,
        new_pos: u8,
        piece: Piece,
        captured_piece: Option<Piece>,
    },
    // Capture{prev_pos: u8, new_pos: u8, piece: Piece, captured_piece: Piece},
    Castle {
        is_short: bool,
    },
    PawnPromote {
        prev_pos: u8,
        new_pos: u8,
        promoted_to_piece: Piece,
        captured_piece: Option<Piece>,
    },
}

impl Move {
    fn get_cmp_key(self, last_move_pos: u8) -> (u8, i32) {
        match self {
            Move::Move {
                new_pos,
                piece,
                captured_piece,
                ..
            } => {
                if let Some(captured_piece) = captured_piece {
                    if captured_piece == Piece::King {
                        (0, 0)
                    } else if new_pos == last_move_pos {
                        (1, piece.value() - captured_piece.value())
                    } else {
                        (3, piece.value() - captured_piece.value())
                    }
                } else {
                    (4, 0)
                }
            }
            Move::Castle { .. } => (4, 0),
            Move::PawnPromote {
                promoted_to_piece, ..
            } => (2, -promoted_to_piece.value()),
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match *self {
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece: Some(captured_piece),
            } => {
                format!(
                    "{} {} to {}, capturing {}",
                    piece,
                    util::coord_to_canonical(util::pos_to_coord(prev_pos)),
                    util::coord_to_canonical(util::pos_to_coord(new_pos)),
                    captured_piece
                )
            }
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece: None,
            } => {
                format!(
                    "{} {} to {}",
                    piece,
                    util::coord_to_canonical(util::pos_to_coord(prev_pos)),
                    util::coord_to_canonical(util::pos_to_coord(new_pos))
                )
            }
            Move::Castle { is_short: true } => "castle short".into(),
            Move::Castle { is_short: false } => "castle long".into(),
            Move::PawnPromote {
                prev_pos,
                new_pos,
                promoted_to_piece,
                captured_piece: Some(captured_piece),
            } => {
                format!(
                    "{} {} to {}, capturing {}, promoting to {}",
                    Piece::Pawn,
                    util::coord_to_canonical(util::pos_to_coord(prev_pos)),
                    util::coord_to_canonical(util::pos_to_coord(new_pos)),
                    captured_piece,
                    promoted_to_piece
                )
            }
            Move::PawnPromote {
                prev_pos,
                new_pos,
                promoted_to_piece,
                captured_piece: None,
            } => {
                format!(
                    "{} {} to {}, promoting to {}",
                    Piece::Pawn,
                    util::coord_to_canonical(util::pos_to_coord(prev_pos)),
                    util::coord_to_canonical(util::pos_to_coord(new_pos)),
                    promoted_to_piece
                )
            }
        };
        f.write_str(&result)
    }
}

#[derive(PartialEq, Eq)]
struct ValueMovePair(i32, Move);

impl PartialOrd for ValueMovePair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for ValueMovePair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

pub struct ChessEngine<
    const MAX_DEPTH_SOFT: usize,
    const MAX_DEPTH_CAPTURE: usize,
    const MAX_DEPTH_HARD: usize,
    const CACHE_MAX_SIZE: usize
> {
    // state: GameState,
    move_buf: [Vec<Move>; MAX_DEPTH_HARD],
    opp_move_grid: u64,
    nodes_explored: usize,
    calculated_moves: BTreeSet<ValueMovePair>,
    zoborist_state: ZoboristState,
    state_cache: HashMap<HASH_TYPE, (usize, f32), FxBuildHasher>
}

// impl<const MAX_DEPTH_SOFT: usize, const MAX_DEPTH_CAPTURE: usize, const MAX_DEPTH_HARD: usize>
//     Default for ChessEngine<MAX_DEPTH_SOFT, MAX_DEPTH_CAPTURE, MAX_DEPTH_HARD>
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl<const MAX_DEPTH_SOFT: usize, const MAX_DEPTH_CAPTURE: usize, const MAX_DEPTH_HARD: usize, const CACHE_MAX_SIZE: usize>
    ChessEngine<MAX_DEPTH_SOFT, MAX_DEPTH_CAPTURE, MAX_DEPTH_HARD,CACHE_MAX_SIZE>
{
    pub fn new(zoborist_state_seed: u64) -> Self {
        if MAX_DEPTH_SOFT > MAX_DEPTH_CAPTURE || MAX_DEPTH_CAPTURE > MAX_DEPTH_HARD {
            panic!("invalid depth parameters");
        }
        const BUF: Vec<Move> = vec![];
        ChessEngine {
            opp_move_grid: 0,
            move_buf: [BUF; MAX_DEPTH_HARD],
            nodes_explored: 0,
            calculated_moves: BTreeSet::default(),
            zoborist_state: ZoboristState::new(zoborist_state_seed),
            state_cache: HashMap::with_hasher(FxBuildHasher::default())
        }
    }

    // TYPE: whether to save it to teh move buffer or to put i
    fn emit_move<const TYPE: bool>(&mut self, next_move: Move, player: Player, depth: usize) {
        if TYPE {
            if let Move::Move {
                prev_pos,
                new_pos,
                captured_piece,
                piece,
            } = next_move
            {
                if piece == Piece::Pawn
                    && ((player == Player::White && (new_pos >> 3) == 7)
                        || (player == Player::Black && (new_pos >> 3) == 0))
                {
                    const PIECES: [Piece; 4] =
                        [Piece::Queen, Piece::Knight, Piece::Rook, Piece::Bishop];
                    for promoted_to_piece in PIECES {
                        self.move_buf[depth].push(Move::PawnPromote {
                            prev_pos,
                            new_pos,
                            promoted_to_piece,
                            captured_piece,
                        });
                    }
                    return;
                }
            }
            self.move_buf[depth].push(next_move);
        } else if let Move::Move { new_pos, .. } = next_move {
            // opp move grid is for player squares that are under attack, is used to check if king can castle
            // note that we can count for pawn moves and pawn captures: this doesn't matter
            self.opp_move_grid |= 1u64 << new_pos;
        }
    }

    fn get_moves_by_deltas<const TYPE: bool, T: IntoIterator<Item = (i8, i8)>>(
        &mut self,
        state: &GameState,
        prev_pos: u8,
        player: Player,
        piece: Piece,
        deltas: T,
        depth: usize,
    ) {
        for (dy, dx) in deltas {
            let mut res_coord = util::pos_to_coord(prev_pos);
            res_coord.0 += dy;
            res_coord.1 += dx;
            while res_coord.0 >= 0 && res_coord.0 < 8 && res_coord.1 >= 0 && res_coord.1 < 8 {
                let new_pos = util::coord_to_pos(res_coord);
                if state.get_state(player).square_occupied(new_pos) {
                    break;
                }
                // self.emit_move::<TYPE>(prev_pos, new_pos, piece, MoveType::MoveCapture, depth);
                // self.move_buf.push((pos, res_pos, MoveType::MoveCapture));
                if let Some(captured_piece) =
                    state.get_state(player.opp()).get_piece_at_pos(new_pos)
                {
                    self.emit_move::<TYPE>(
                        Move::Move {
                            prev_pos,
                            new_pos,
                            piece,
                            captured_piece: Some(captured_piece),
                        },
                        player,
                        depth,
                    );
                    break;
                }
                self.emit_move::<TYPE>(
                    Move::Move {
                        prev_pos,
                        new_pos,
                        piece,
                        captured_piece: None,
                    },
                    player,
                    depth,
                );
                res_coord.0 += dy;
                res_coord.1 += dx;
            }
        }
    }

    // todo: enpassant, double move
    fn get_move_by_piece<const TYPE: bool>(
        &mut self,
        state: &GameState,
        piece: Piece,
        prev_pos: u8,
        player: Player,
        depth: usize,
    ) {
        match piece {
            Piece::Pawn => {
                if player == Player::White {
                    if !state.get_state(player.opp()).square_occupied(prev_pos + 8) {
                        self.emit_move::<TYPE>(
                            Move::Move {
                                prev_pos,
                                new_pos: prev_pos + 8,
                                piece,
                                captured_piece: None,
                            },
                            player,
                            depth,
                        );
                        if (prev_pos >> 3) == 1
                            && !state.get_state(player.opp()).square_occupied(prev_pos + 16)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos + 16,
                                    piece,
                                    captured_piece: None,
                                },
                                player,
                                depth,
                            );
                        }
                    }

                    if (prev_pos & 0b111) > 0 {
                        if let Some(captured_piece) =
                            state.get_state(player.opp()).get_piece_at_pos(prev_pos + 7)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos + 7,
                                    piece,
                                    captured_piece: Some(captured_piece),
                                },
                                player,
                                depth,
                            );
                        }
                    }
                    if (prev_pos & 0b111) < 7 {
                        if let Some(captured_piece) =
                            state.get_state(player.opp()).get_piece_at_pos(prev_pos + 9)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos + 9,
                                    piece,
                                    captured_piece: Some(captured_piece),
                                },
                                player,
                                depth,
                            );
                        }
                    }
                } else {
                    if !state.get_state(player.opp()).square_occupied(prev_pos - 8) {
                        self.emit_move::<TYPE>(
                            Move::Move {
                                prev_pos,
                                new_pos: prev_pos - 8,
                                piece,
                                captured_piece: None,
                            },
                            player,
                            depth,
                        );
                        if (prev_pos >> 3) == 6
                            && !state.get_state(player.opp()).square_occupied(prev_pos - 16)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos - 16,
                                    piece,
                                    captured_piece: None,
                                },
                                player,
                                depth,
                            );
                        }
                    }
                    if (prev_pos & 0b111) > 0 {
                        if let Some(captured_piece) =
                            state.get_state(player.opp()).get_piece_at_pos(prev_pos - 9)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos - 9,
                                    piece,
                                    captured_piece: Some(captured_piece),
                                },
                                player,
                                depth,
                            );
                        }
                    }
                    if (prev_pos & 0b111) < 7 {
                        if let Some(captured_piece) =
                            state.get_state(player.opp()).get_piece_at_pos(prev_pos - 7)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos - 7,
                                    piece,
                                    captured_piece: Some(captured_piece),
                                },
                                player,
                                depth,
                            );
                        }
                    }
                }
            }
            Piece::Knight => {
                const KNIGHT_DELTAS: [(i8, i8); 8] = [
                    (-1, -2),
                    (-1, 2),
                    (1, -2),
                    (1, 2),
                    (-2, -1),
                    (-2, 1),
                    (2, -1),
                    (2, 1),
                ];
                let coord = util::pos_to_coord(prev_pos);
                for (dy, dx) in KNIGHT_DELTAS {
                    let res_coord = (coord.0 + dy, coord.1 + dx);
                    let new_pos = util::coord_to_pos(res_coord);
                    if res_coord.0 >= 0
                        && res_coord.0 < 8
                        && res_coord.1 >= 0
                        && res_coord.1 < 8
                        && !state.get_state(player).square_occupied(new_pos)
                    {
                        self.emit_move::<TYPE>(
                            Move::Move {
                                prev_pos,
                                new_pos,
                                piece,
                                captured_piece: state
                                    .get_state(player.opp())
                                    .get_piece_at_pos(new_pos),
                            },
                            player,
                            depth,
                        );
                    }
                }
            }
            Piece::Bishop => {
                const BISHOP_DELTAS: [(i8, i8); 4] = [(1, -1), (1, 1), (-1, -1), (-1, 1)];
                self.get_moves_by_deltas::<TYPE, _>(
                    state,
                    prev_pos,
                    player,
                    Piece::Bishop,
                    BISHOP_DELTAS,
                    depth,
                );
            }
            Piece::Rook => {
                const ROOK_DELTAS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, -1), (0, 1)];
                self.get_moves_by_deltas::<TYPE, _>(
                    state,
                    prev_pos,
                    player,
                    Piece::Rook,
                    ROOK_DELTAS,
                    depth,
                );
            }
            Piece::Queen => {
                const QUEEN_DELTAS: [(i8, i8); 8] = [
                    (1, -1),
                    (1, 1),
                    (-1, -1),
                    (-1, 1),
                    (1, 0),
                    (-1, 0),
                    (0, -1),
                    (0, 1),
                ];
                self.get_moves_by_deltas::<TYPE, _>(
                    state,
                    prev_pos,
                    player,
                    Piece::Queen,
                    QUEEN_DELTAS,
                    depth,
                );
            }
            Piece::King => {
                const KING_DELTAS: [(i8, i8); 8] = [
                    (1, -1),
                    (1, 1),
                    (-1, -1),
                    (-1, 1),
                    (1, 0),
                    (-1, 0),
                    (0, -1),
                    (0, 1),
                ];
                let coord = util::pos_to_coord(prev_pos);

                for (dy, dx) in KING_DELTAS {
                    let res_coord = (coord.0 + dy, coord.1 + dx);
                    let new_pos = util::coord_to_pos(res_coord);

                    // square is valid
                    // check for square under attack by opposing piece is done during tree search
                    if res_coord.0 >= 0
                        && res_coord.0 < 8
                        && res_coord.1 >= 0
                        && res_coord.1 < 8
                        && state.get_state(player).piece_grid & (1 << new_pos) == 0
                    {
                        self.emit_move::<TYPE>(
                            Move::Move {
                                prev_pos,
                                new_pos,
                                piece,
                                captured_piece: state
                                    .get_state(player.opp())
                                    .get_piece_at_pos(new_pos),
                            },
                            player,
                            depth,
                        );
                    }
                }

                let player_state = state.get_state(player);
                let offset = if player == Player::White { 0 } else { 7 };

                // optimization
                if !player_state.square_occupied(5 + offset)
                    && !player_state.square_occupied(6 + offset)
                    && !player_state.square_occupied(2 + offset)
                    && !player_state.square_occupied(3 + offset)
                    && !player_state.square_occupied(4 + offset)
                    && (player_state.meta.can_castle_short
                        && self.opp_move_grid
                            | (1 << (4 + offset))
                            | (1 << (5 + offset))
                            | (1 << (6 + offset))
                            == 0)
                    || (player_state.meta.can_castle_long
                        && self.opp_move_grid
                            | (1 << (4 + offset))
                            | (1 << (3 + offset))
                            | (1 << (2 + offset))
                            == 0)
                {
                    self.opp_move_grid = 0;
                    self.get_all_moves::<false>(state, state.player.opp(), depth);

                    if !player_state.square_occupied(5 + offset)
                        && !player_state.square_occupied(6 + offset)
                        && player_state.meta.can_castle_short
                        && self.opp_move_grid
                            | (1 << (4 + offset))
                            | (1 << (5 + offset))
                            | (1 << (6 + offset))
                            == 0
                    {
                        self.emit_move::<TYPE>(Move::Castle { is_short: true }, player, depth)
                    }
                    if !player_state.square_occupied(2 + offset)
                        && !player_state.square_occupied(3 + offset)
                        && !player_state.square_occupied(4 + offset)
                        && player_state.meta.can_castle_long
                        && self.opp_move_grid
                            | (1 << (4 + offset))
                            | (1 << (3 + offset))
                            | (1 << (2 + offset))
                            == 0
                    {
                        self.emit_move::<TYPE>(Move::Castle { is_short: false }, player, depth)
                    }
                }
            }
        }
    }
    // overrides move_buf
    // todo: en-passant
    fn get_moves_by_piece_type<const TYPE: bool>(
        &mut self,
        state: &GameState,
        piece: Piece,
        player: Player,
        depth: usize,
    ) {
        for i in 0u8..64 {
            if state.get_state(player).pieces[piece as usize].0 & (1u64 << i) > 0 {
                self.get_move_by_piece::<TYPE>(state, piece, i, player, depth);
            }
        }
    }

    fn get_all_moves<const TYPE: bool>(&mut self, state: &GameState, player: Player, depth: usize) {
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Pawn, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Knight, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Bishop, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Rook, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Queen, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::King, player, depth);
    }

    // fn recurse(
    //     &mut self,
    //     state: &mut GameState,
    //     value: &mut f32,
    //     alpha: &mut f32,
    //     beta: &mut f32,
    //     rem_depth: usize,
    //     last_capture: Option<u8>
    // ) {

    // }
    // returns (score, initial piece pos, move piece pos)
    fn calc(
        &mut self,
        state: &mut GameState,
        mut alpha: f32,
        mut beta: f32,
        depth: usize,
        last_move_pos: u8,
    ) -> f32 {
        self.nodes_explored += 1;
        if self.nodes_explored % 100000 == 0 {
            println!(
                "explored {} nodes, alpha: {}, beta: {}",
                self.nodes_explored, alpha, beta
            );
        }

        if let Some((cached_depth, value)) = self.state_cache.get(&state.hash){
            if *cached_depth <= depth{
                return *value;
            }
        }

        let score = state.score();
        if depth == MAX_DEPTH_HARD {
            return score as f32;
        }

        let mut value = if state.player == Player::White {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        };

        // if depth > MAX_DEPTH_CAPTURE {
        //     return (if value <= -500_000_000. || value >= 500_000_000. {score as f32} else {value}, score as f32)
        // }

        self.move_buf[depth].clear();
        self.get_all_moves::<true>(state, state.player, depth);

        self.move_buf[depth].sort_by_key(|a| Reverse(a.get_cmp_key(last_move_pos)));

        while let Some(next_move) = self.move_buf[depth].pop() {
            if let Move::Move {
                captured_piece: Some(Piece::King),
                ..
            } = next_move
            {
                let result = if state.player == Player::White {
                    500_000_000.
                } else {
                    -500_000_000.
                };
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
                        return if value <= -500_000_000. || value >= 500_000_000. {
                            score as f32
                        } else {
                            value
                        };
                    }
                } else {
                    return if value <= -500_000_000. || value >= 500_000_000. {
                        score as f32
                    } else {
                        value
                    };
                }
                // return if value <= -500_000_000. || value >= 500_000_000. {score as f32} else {value}
            }

            if depth > MAX_DEPTH_CAPTURE {
                // return (if value <= -500_000_000. || value >= 500_000_000. {score as f32} else {value}, score as f32)
                // return (0., 0.)
                // break;
            }

            // let last_state = state.clone();

            let meta = (state.states[0].meta, state.states[1].meta);
            state.apply_meta_hash(&self.zoborist_state);
            state.advance_state(next_move, &self.zoborist_state);
            state.apply_meta_hash(&self.zoborist_state);

            // update meta values
            let next_val = self.calc(
                state,
                alpha,
                beta,
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

            if state.player == Player::White {
                value = f32::max(value, next_val);
                if value >= beta {
                    break;
                }
                alpha = f32::max(alpha, value);
            } else {
                value = f32::min(value, next_val);
                if value <= alpha {
                    break;
                }
                beta = f32::min(beta, value);
            }

            if depth == 0 {
                self.calculated_moves
                    .insert(ValueMovePair(next_val as i32, next_move));
            }
        }

        self.state_cache.insert(state.hash, (depth, value));

        value
    }

    pub fn solve(&mut self, state: &mut GameState) -> f32 {
        state.slow_compute_hash(&self.zoborist_state);
        self.calc(state, f32::NEG_INFINITY, f32::INFINITY, 0, 0)
    }

    pub fn get_moves(&mut self) {
        while let Some(ValueMovePair(value, m)) = self.calculated_moves.pop_last() {
            println!("{}: {}", value, m);
        }
    }
}
