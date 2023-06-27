use std::cmp::Reverse;

use crate::{
    config::MAX_CHILDREN_PER_NODE,
    move_orderer::MoveOrderer,
    types::{Move, Piece},
    util, GameState, Player,
};

pub struct MoveBuffer {
    opp_move_grid: u64,
    num_moves: usize,
    move_buf: [Option<Move>; MAX_CHILDREN_PER_NODE],
    // move_buf: Vec<Vec<Option<Move>>>,
}

impl Default for MoveBuffer {
    fn default() -> Self {
        Self {
            opp_move_grid: Default::default(),
            num_moves: 0,
            move_buf: [None; MAX_CHILDREN_PER_NODE],
        }
    }
}

impl MoveBuffer {
    // TYPE: whether to save it to teh move buffer or to put i
    fn emit_move<const TYPE: bool>(&mut self, next_move: Move, player: Player) {
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
                        self.move_buf[self.num_moves] = Some(Move::PawnPromote {
                            prev_pos,
                            new_pos,
                            promoted_to_piece,
                            captured_piece,
                        });
                        self.num_moves += 1;
                    }
                    return;
                }
            }
            self.move_buf[self.num_moves] = Some(next_move);
            self.num_moves += 1;
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
                );
                res_coord.0 += dy;
                res_coord.1 += dx;
            }
        }
    }

    // todo: enpassant
    pub fn get_move_by_piece<const TYPE: bool>(
        &mut self,
        state: &GameState,
        piece: Piece,
        prev_pos: u8,
        player: Player,
    ) {
        match piece {
            Piece::Pawn => {
                if player == Player::White {
                    if !state.get_state(player.opp()).square_occupied(prev_pos + 8)
                        && !state.get_state(player).square_occupied(prev_pos + 8)
                    {
                        self.emit_move::<TYPE>(
                            Move::Move {
                                prev_pos,
                                new_pos: prev_pos + 8,
                                piece,
                                captured_piece: None,
                            },
                            player,
                        );
                        if (prev_pos >> 3) == 1
                            && !state.get_state(player.opp()).square_occupied(prev_pos + 16)
                            && !state.get_state(player).square_occupied(prev_pos + 16)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos + 16,
                                    piece,
                                    captured_piece: None,
                                },
                                player,
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
                            );
                        }
                    }
                } else {
                    if !state.get_state(player.opp()).square_occupied(prev_pos - 8)
                        && !state.get_state(player).square_occupied(prev_pos - 8)
                    {
                        self.emit_move::<TYPE>(
                            Move::Move {
                                prev_pos,
                                new_pos: prev_pos - 8,
                                piece,
                                captured_piece: None,
                            },
                            player,
                        );
                        if (prev_pos >> 3) == 6
                            && !state.get_state(player.opp()).square_occupied(prev_pos - 16)
                            && !state.get_state(player).square_occupied(prev_pos - 16)
                        {
                            self.emit_move::<TYPE>(
                                Move::Move {
                                    prev_pos,
                                    new_pos: prev_pos - 16,
                                    piece,
                                    captured_piece: None,
                                },
                                player,
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
                    if res_coord.0 >= 0
                        && res_coord.0 < 8
                        && res_coord.1 >= 0
                        && res_coord.1 < 8
                        && !state
                            .get_state(player)
                            .square_occupied(util::coord_to_pos(res_coord))
                    {
                        let new_pos = util::coord_to_pos(res_coord);
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
                        );
                    }
                }

                let player_state = state.get_state(player);
                let offset = if player == Player::White { 0 } else { 56 };

                // optimization
                if TYPE
                    && ((!player_state.square_occupied(5 + offset)
                        && !player_state.square_occupied(6 + offset)
                        && player_state.meta.can_castle_short)
                        || (!player_state.square_occupied(1 + offset)
                            && !player_state.square_occupied(2 + offset)
                            && !player_state.square_occupied(3 + offset)
                            && player_state.meta.can_castle_long))
                {
                    self.opp_move_grid = 0;
                    self.get_all_moves::<false>(state, state.player.opp());

                    if !player_state.square_occupied(5 + offset)
                        && !player_state.square_occupied(6 + offset)
                        && player_state.meta.can_castle_short
                        && self.opp_move_grid
                            & ((1 << (4 + offset)) | (1 << (5 + offset)) | (1 << (6 + offset)))
                            == 0
                    {
                        self.emit_move::<TYPE>(Move::Castle { is_short: true }, player)
                    }
                    if !player_state.square_occupied(1 + offset)
                        && !player_state.square_occupied(2 + offset)
                        && !player_state.square_occupied(3 + offset)
                        && player_state.meta.can_castle_long
                        && self.opp_move_grid
                            & ((1 << (4 + offset)) | (1 << (3 + offset)) | (1 << (2 + offset)))
                            == 0
                    {
                        self.emit_move::<TYPE>(Move::Castle { is_short: false }, player)
                    }
                }
            }
        }
    }
    // overrides move_buf
    // todo: en-passant
    pub fn get_moves_by_piece_type<const TYPE: bool>(
        &mut self,
        state: &GameState,
        piece: Piece,
        player: Player,
    ) {
        for i in 0u8..64 {
            if state.get_state(player).pieces[piece as usize].0 & (1u64 << i) > 0 {
                self.get_move_by_piece::<TYPE>(state, piece, i, player);
            }
        }
    }

    pub fn get_all_moves<const TYPE: bool>(&mut self, state: &GameState, player: Player) {
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Pawn, player);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Knight, player);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Bishop, player);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Rook, player);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Queen, player);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::King, player);
    }

    // pub fn sort(&mut self, last_move_pos: u8, killer_entry: KillerEntry){
    //     // TODO: compare new method to old method
    //     self.move_buf[0..self.num_moves].sort_by_key(|a| Reverse(a.unwrap().get_cmp_key(last_move_pos, killer_entry)));
    // }

    pub fn pop(&mut self) -> Option<Move> {
        if self.num_moves == 0 {
            None
        } else {
            self.num_moves -= 1;
            self.move_buf[self.num_moves]
        }
        // self.move_buf[depth].pop()?
    }

    pub fn get_next_move(
        &mut self,
        last_move_pos: u8,
        depth: usize,
        player: Player,
        transposition_move: Option<Move>,
        move_orderer: &mut MoveOrderer,
    ) -> Option<Move> {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .filter_map(|x| if x.is_some() { Some(x) } else { None })
            .min_by(|x, y| {
                // x.unwrap()
                //     .get_cmp_key(last_move_pos, killer_entry)
                //     .cmp(&y.unwrap().get_cmp_key(last_move_pos, killer_entry))
                move_orderer.cmp_move(x.unwrap(), y.unwrap(), depth, last_move_pos, player, transposition_move)
            })?
            .take()
    }

    pub fn clear(&mut self) {
        self.num_moves = 0;
    }

    // checks if the opposing king can be captured
    pub fn is_legal(&self) -> bool {
        !self.move_buf[0..self.num_moves]
            .iter()
            .filter_map(|x| *x)
            .any(|x| match x {
                Move::Move {
                    captured_piece: Some(captured_piece),
                    ..
                } => captured_piece == Piece::King,
                Move::PawnPromote {
                    captured_piece: Some(captured_piece),
                    ..
                } => captured_piece == Piece::King,
                _ => false,
            })
    }

    // TODO: improve
    pub fn is_zugzwang(&self) -> bool {
        self.num_moves == 0
    }
}
