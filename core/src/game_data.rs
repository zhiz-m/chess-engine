use serde::Deserialize;

use crate::grid::PieceGrid;
use crate::markers::{player_to_marker, CastleTypeMarker, PlayerMarker};
use crate::player::Player;
use crate::square_type::SquareType;
use crate::types_for_io::Piece;
use crate::zoborist_state::ZoboristState;
use crate::{config::HashType, grid::Grid, types::Move, util::coord_to_pos};
use std::hash::Hash;

// cheap, copyable player state
// #[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
// pub struct PlayerMetadata {
//     pub can_castle_short: bool,
//     pub can_castle_long: bool,
// }

// impl Default for PlayerMetadata {
//     fn default() -> Self {
//         Self {
//             can_castle_short: true,
//             can_castle_long: true,
//         }
//     }
// }

#[derive(PartialEq, Eq, Clone, Copy, Deserialize)]
pub struct Metadata(u8);

impl Default for Metadata {
    fn default() -> Self {
        // no en passant, can castle
        Self(0b10001111)
    }
}

impl Metadata {
    pub const NO_EN_PASSANT: u8 = 8;

    #[inline(always)]
    const fn castle_bits<P: PlayerMarker, C: CastleTypeMarker>() -> u8 {
        if C::IS_SHORT {
            if P::IS_WHITE {
                0b1
            } else {
                0b10
            }
        } else {
            if P::IS_WHITE {
                0b100
            } else {
                0b1000
            }
        }
    }

    #[inline(always)]
    pub fn get_can_castle<P: PlayerMarker, C: CastleTypeMarker>(self) -> bool {
        self.0 & Self::castle_bits::<P, C>() > 0
    }

    #[inline(always)]
    pub fn set_can_castle<P: PlayerMarker, C: CastleTypeMarker, const B: bool>(&mut self) {
        if B {
            self.0 |= Self::castle_bits::<P, C>();
        } else {
            self.0 &= !Self::castle_bits::<P, C>();
        }
    }

    #[inline(always)]
    fn castle_bits_dynamic(player: Player, is_short: bool) -> u8 {
        let is_short = !is_short as u8;
        ((1 << is_short) << is_short) << player as u8
    }

    #[inline(always)]
    pub fn get_can_castle_dynamic(self, player: Player, is_short: bool) -> bool {
        self.0 & Self::castle_bits_dynamic(player, is_short) > 0
    }

    #[inline(always)]
    pub fn set_can_castle_dynamic<const B: bool>(&mut self, player: Player, is_short: bool) {
        if B {
            self.0 |= Self::castle_bits_dynamic(player, is_short);
        } else {
            self.0 &= !Self::castle_bits_dynamic(player, is_short);
        }
    }

    #[inline(always)]
    pub fn get_en_passant_column(self) -> u8 {
        self.0 >> 4
    }

    #[inline(always)]
    pub fn set_en_passant_column(&mut self, en_passant_column: u8) {
        self.0 = (self.0 & 0b1111) | (en_passant_column << 4)
    }

    #[inline(always)]
    pub fn get_meta_hash(self, zoborist_state: &ZoboristState) -> HashType {
        zoborist_state.castle[(self.0 & 0b1111) as usize]
            ^ zoborist_state.en_passant[(self.0 >> 4) as usize]
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct GameState {
    pub piece_grid: PieceGrid,
    pub metadata: Metadata,
    pub player: Player,
    #[serde(default)]
    pub hash: HashType,
}

impl Hash for GameState {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            piece_grid: PieceGrid::from_piece_vec(
                &[
                    vec!["a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2"],
                    vec!["b1", "g1"],
                    vec!["c1", "f1"],
                    vec!["a1", "h1"],
                    vec!["d1"],
                    vec!["e1"],
                ],
                &[
                    vec!["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7"],
                    vec!["b8", "g8"],
                    vec!["c8", "f8"],
                    vec!["a8", "h8"],
                    vec!["d8"],
                    vec!["e8"],
                ],
            ),
            metadata: Metadata::default(),
            player: Player::White,
            hash: 0,
        }
    }
}

impl GameState {
    pub fn new_with_hash(zoborist_state: &ZoboristState) -> Self{
        let mut res = Self::default();
        res.setup(zoborist_state);
        res
    }

    // todo: maybe relax legality checking for performance
    pub fn check_move_legal(&self, mov: Move) -> bool {
        match mov {
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece,
            } => {
                self.piece_grid
                    .get_squares_of_type(piece)
                    .all_squares_occupied(Grid::from_pos(prev_pos))
                    && self
                        .piece_grid
                        .get_squares_of_type(captured_piece)
                        .all_squares_occupied(Grid::from_pos(new_pos))
            }
            Move::Castle { is_short } => {
                self.metadata.get_can_castle_dynamic(self.player, is_short)
            }
            Move::PawnPromote {
                prev_pos,
                new_pos,
                captured_piece,
                ..
            } => {
                player_to_marker!(self.player, { self.piece_grid.get_pawn_pos::<P>() })
                    .all_squares_occupied(Grid::from_pos(prev_pos))
                    && self
                        .piece_grid
                        .get_squares_of_type(captured_piece)
                        .all_squares_occupied(Grid::from_pos(new_pos))
            }
            // todo: maybe make this better
            Move::EnPassant { new_column, .. } => {
                self.metadata.get_en_passant_column() == new_column
            }
        }
    }

    fn apply_fen(
        piece_grid: &mut PieceGrid,
        metadata: &mut Metadata,
        player: Player,
        fen: &str,
    ) -> Result<(), &'static str> {
        if !fen.is_ascii() {
            return Err("fen not ascii");
        }

        let parts: Vec<&str> = fen.split(' ').collect();

        if parts.len() != 6 {
            return Err("fen not len 6");
        }

        // handle pieces

        let mut row = 7;
        let mut column = 7;

        let char_is_valid = |char: char| {
            (player == Player::White && char.is_ascii_uppercase())
                || (player == Player::Black && char.is_ascii_lowercase())
        };

        for char in parts[0].chars() {
            if char == '/' {
                row -= 1;
                column = 7;
                continue;
            }
            if row < 0 || column < 0 {
                return Err("invalid fen");
            }
            if let Some(num) = char.to_digit(10) {
                column -= num as i8;
                continue;
            }

            if char_is_valid(char) {
                let piece = Piece::try_from(char)?;
                let pos = coord_to_pos((row, column));
                println!("{} {}", piece, pos);

                piece_grid.apply_square(pos, SquareType::create_for_parsing(piece, player))
            }
            column -= 1
        }

        metadata.set_can_castle_dynamic::<false>(player, true);
        metadata.set_can_castle_dynamic::<false>(player, false);

        // handle castling
        for char in parts[2].chars() {
            if char == '-' {
                break;
            }
            if char_is_valid(char) {
                match char.to_ascii_lowercase() {
                    'k' => metadata.set_can_castle_dynamic::<true>(player, true),
                    'q' => metadata.set_can_castle_dynamic::<true>(player, false),
                    _ => return Err("invalid castle char"),
                }
            }
        }

        // todo: everything else

        // state.setup();

        Ok(())
    }

    pub fn new_from_fen(fen: &str, zoborist_state: &ZoboristState) -> Result<Self, &'static str> {
        let parts: Vec<_> = fen.split(' ').collect();
        let mut piece_grid = PieceGrid::default();
        let mut metadata = Metadata::default();
        Self::apply_fen(&mut piece_grid, &mut metadata, Player::White, fen)?;
        Self::apply_fen(&mut piece_grid, &mut metadata, Player::Black, fen)?;
        metadata.set_en_passant_column(match parts[3].chars().nth(0).ok_or("invalid fen")? {
            '-' => 8,
            x if ('a'..='h').contains(&x) => x as u8 - 'a' as u8,
            _ => return Err("invalid en passant value in fen"),
        });

        // println!("metadata: {:b}", metadata.0);
        let mut res = Self {
            piece_grid,
            metadata,
            player: Player::try_from(parts[1].chars().nth(0).ok_or("invalid fen")?)?,
            hash: 0,
        };
        res.setup(zoborist_state);

        Ok(res)
    }

    #[inline(always)]
    pub fn apply_meta_hash(&mut self, zoborist_state: &ZoboristState) {
        self.hash ^= self.metadata.get_meta_hash(zoborist_state)
    }

    #[inline(always)]
    fn apply_piece_hash(
        &mut self,
        zoborist_state: &ZoboristState,
        square_type: SquareType,
        pos: u8,
    ) {
        self.hash ^= square_type.get_piece_hash(pos, zoborist_state)
    }

    #[inline(always)]
    pub fn change_player(&mut self, zoborist_state: &ZoboristState) {
        self.hash ^= zoborist_state.player;
        self.player = self.player.opp();
    }

    #[inline(always)]
    fn apply_piece_move(
        &mut self,
        zoborist_state: &ZoboristState,
        square_type: SquareType,
        pos: u8,
    ) {
        self.apply_piece_hash(zoborist_state, square_type, pos);

        self.piece_grid.apply_square(pos, square_type);
    }

    // zoborist hash
    fn slow_compute_hash(&mut self, zoborist_state: &ZoboristState) {
        self.apply_meta_hash(zoborist_state);
        if self.player == Player::Black {
            self.hash ^= zoborist_state.player;
        }
        for pos in 0..64u8 {
            let square_type = self.piece_grid.get_square_type(pos);
            self.apply_piece_hash(zoborist_state, square_type, pos);
        }
    }

    pub fn setup(&mut self, zoborist_state: &ZoboristState) {
        self.slow_compute_hash(zoborist_state);
    }

    // pub fn get_state(&self, player: Player) -> &PlayerState {
    //     &self.states[player as usize]
    // }

    // fn get_state_mut(&mut self, player: Player) -> &mut PlayerState {
    //     &mut self.states[player as usize]
    // }

    #[inline(always)]
    fn modify_state<const APPLY_METADATA_CHANGES: bool>(
        &mut self,
        next_move: Move,
        zoborist_state: &ZoboristState,
    ) {
        if APPLY_METADATA_CHANGES {
            self.metadata.set_en_passant_column(Metadata::NO_EN_PASSANT);
            // self.en_passant_column = None;
        }
        let offset = self.player as u8 * 56;
        match next_move {
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece,
            } => {
                self.apply_piece_move(zoborist_state, captured_piece, new_pos);

                self.apply_piece_move(zoborist_state, piece, prev_pos);
                self.apply_piece_move(zoborist_state, piece, new_pos);

                if APPLY_METADATA_CHANGES {
                    if piece.is_king() || (piece.is_rook() && prev_pos == offset + 7) {
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player, false);
                    }
                    if piece.is_king() || (piece.is_rook() && prev_pos == offset) {
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player, true);
                    }
                    let offset_opp = (1 - self.player as u8) * 56;
                    if captured_piece.is_rook() && new_pos == offset_opp {
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player.opp(), true);
                    }
                    if captured_piece.is_rook() && new_pos == offset_opp + 7 {
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player.opp(), false);
                    }

                    // potential en passant next move
                    if piece.is_pawn() && i32::abs(prev_pos as i32 - new_pos as i32) == 16 {
                        self.metadata.set_en_passant_column(prev_pos & 0b111);
                    }
                }
            }
            Move::Castle { is_short } => {
                const KING_POS: u8 = 3;
                if APPLY_METADATA_CHANGES {
                    self.metadata
                        .set_can_castle_dynamic::<false>(self.player, false);
                    self.metadata
                        .set_can_castle_dynamic::<false>(self.player, true);
                }

                let rook_pos = if is_short { 0 } else { 7 };

                let king = SquareType::king(self.player);
                let rook = SquareType::rook(self.player);

                self.apply_piece_move(zoborist_state, king, KING_POS + offset);
                self.apply_piece_move(zoborist_state, rook, rook_pos + offset);

                // xor with post-castling rook/king positions
                if is_short {
                    self.apply_piece_move(zoborist_state, king, KING_POS + offset - 2);
                    self.apply_piece_move(zoborist_state, rook, rook_pos + offset + 2);
                } else {
                    self.apply_piece_move(zoborist_state, king, KING_POS + offset + 2);
                    self.apply_piece_move(zoborist_state, rook, rook_pos + offset - 3);
                }
            }
            Move::PawnPromote {
                prev_pos,
                new_pos,
                promoted_to_piece,
                captured_piece,
            } => {
                self.apply_piece_move(zoborist_state, captured_piece, new_pos);

                // pawn promotion
                self.apply_piece_move(zoborist_state, SquareType::pawn(self.player), prev_pos);
                self.apply_piece_move(zoborist_state, promoted_to_piece, new_pos);
                if APPLY_METADATA_CHANGES {
                    let offset_opp = (1 - self.player as u8) * 56;
                    if captured_piece.is_rook() && new_pos == offset_opp {
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player.opp(), true);
                    }
                    if captured_piece.is_rook() && new_pos == offset_opp + 7 {
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player.opp(), false);
                    }
                }
            }
            Move::EnPassant {
                prev_column,
                new_column,
            } => {
                // todo: remove branches using bitshifts
                let captured_pawn_pos = if self.player == Player::White {
                    32 + new_column
                } else {
                    24 + new_column
                };
                let prev_pos = if self.player == Player::White {
                    32 + prev_column
                } else {
                    24 + prev_column
                };
                let new_pos = if self.player == Player::White {
                    40 + new_column
                } else {
                    16 + new_column
                };
                self.apply_piece_move(
                    zoborist_state,
                    SquareType::pawn(self.player.opp()),
                    captured_pawn_pos,
                );

                // move the current piece
                self.apply_piece_move(zoborist_state, SquareType::pawn(self.player), prev_pos);
                self.apply_piece_move(zoborist_state, SquareType::pawn(self.player), new_pos);
            }
        }
    }

    // #[inline(always)]
    pub fn advance_state(&mut self, next_move: Move, zoborist_state: &ZoboristState) {
        self.modify_state::<true>(next_move, zoborist_state);
        // switch the player
        self.change_player(zoborist_state);
    }

    pub fn advance_state_no_metadata_update(
        &mut self,
        next_move: Move,
        zoborist_state: &ZoboristState,
    ) {
        self.modify_state::<false>(next_move, zoborist_state);
        // switch the player
        self.change_player(zoborist_state);
    }

    // #[inline(always)]
    pub fn revert_state(&mut self, next_move: Move, zoborist_state: &ZoboristState) {
        // switch the player
        self.change_player(zoborist_state);

        self.modify_state::<false>(next_move, zoborist_state);
    }
}
