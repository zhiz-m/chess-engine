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
        self.0 = (self.0 & 0b1111) | en_passant_column
    }

    #[inline(always)]
    pub fn get_meta_hash(self, zoborist_state: &ZoboristState) -> HashType {
        zoborist_state.castle[(self.0 & 0b1111) as usize]
            ^ zoborist_state.en_passant[(self.0 >> 4) as usize]
    }
}

// #[derive(Clone, Deserialize, Default, PartialEq, Eq)]
// pub struct PlayerState {
//     // #[serde(deserialize_with = "from_string_vec")]
//     // pub pieces: [Grid; NUM_PIECES],
//     #[serde(default)]
//     pub pieces: PieceGrid,

//     // pawn: a & !b & c
//     // knight: !a & b & c
//     // bishop: !a & b & c
//     // rook: a & !b & !c
//     // queen: a & b & !c
//     // king: a & b & c

//     // #[serde(default)]
//     // pub piece_grid: u64,
//     #[serde(default)]
//     pub meta: PlayerMetadata,
// }

// fn from_string_vec<'de, D>(deserializer: D) -> Result<[Grid; NUM_PIECES], D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let pieces: BTreeMap<String, Vec<String>> = Deserialize::deserialize(deserializer)?;

//     let mut res = [Grid(0); NUM_PIECES];

//     for (piece, pos) in pieces.into_iter() {
//         let piece: Piece = piece.as_str().try_into().map_err(D::Error::custom)?;
//         let grid: Grid = pos.iter().map(|x| x.as_str()).into();
//         res[piece as usize] = grid;
//     }

//     Ok(res)
// }

// impl PlayerState {
//     pub fn new(player: Player) -> PlayerState {
//         PlayerState {
//             pieces: From::<&[Grid]>::from(&match player {
//                 Player::White => [
//                     vec!["a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2"].into(),
//                     vec!["b1", "g1"].into(),
//                     vec!["c1", "f1"].into(),
//                     vec!["a1", "h1"].into(),
//                     vec!["d1"].into(),
//                     vec!["e1"].into(),
//                 ],
//                 Player::Black => [
//                     vec!["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7"].into(),
//                     vec!["b8", "g8"].into(),
//                     vec!["c8", "f8"].into(),
//                     vec!["a8", "h8"].into(),
//                     vec!["d8"].into(),
//                     vec!["e8"].into(),
//                 ],
//             }),
//             // piece_grid: 0xffff00000000ffff,
//             meta: PlayerMetadata::default(),
//         }
//     }

//     pub fn new_from_state<S: AsRef<str>>(pos: Vec<Vec<S>>) -> Result<PlayerState, &'static str> {
//         if pos.len() != 6 {
//             return Err("invalid length");
//         }
//         let pieces: [Grid; NUM_PIECES] = pos
//             .into_iter()
//             .map(Into::<Grid>::into)
//             .collect::<Vec<Grid>>()
//             .try_into()
//             .map_err(|_| "cannot parse strings")?;
//         let mut res = PlayerState {
//             pieces: PieceGrid::from(pieces.as_slice()),
//             // piece_grid: 0,
//             meta: PlayerMetadata::default(),
//         };
//         // res.setup();

//         Ok(res)
//     }
//     pub fn new_from_fen(player: Player, fen: &str) -> Result<PlayerState, &'static str> {
//         if !fen.is_ascii() {
//             return Err("fen not ascii");
//         }

//         let parts: Vec<&str> = fen.split(' ').collect();

//         if parts.len() != 6 {
//             return Err("fen not len 6");
//         }

//         let mut pieces = [Grid::default(); 6];
//         let mut state = Self::default();

//         // handle pieces

//         let mut row = 7;
//         let mut column = 0;

//         let char_is_valid = |char: char| {
//             (player == Player::White && char.is_ascii_uppercase())
//                 || (player == Player::Black && char.is_ascii_lowercase())
//         };

//         for char in parts[0].chars() {
//             if char == '/' {
//                 row -= 1;
//                 column = 0;
//                 continue;
//             }
//             if row < 0 || column > 7 {
//                 return Err("invalid fen");
//             }
//             if let Some(num) = char.to_digit(10) {
//                 column += num as i8;
//                 continue;
//             }

//             if char_is_valid(char) {
//                 let piece = Piece::try_from(char)?;
//                 let pos = coord_to_pos((row, column));

//                 pieces[piece as usize] |= Grid::from_u64(1 << pos);
//             }
//             column += 1
//         }

//         state.meta.can_castle_long = false;
//         state.meta.can_castle_short = false;

//         // handle castling
//         for char in parts[2].chars() {
//             if char == '-' {
//                 break;
//             }
//             if char_is_valid(char) {
//                 match char.to_ascii_lowercase() {
//                     'k' => state.meta.can_castle_short = true,
//                     'q' => state.meta.can_castle_long = true,
//                     _ => return Err("invalid castle char"),
//                 }
//             }
//         }

//         // todo: everything else

//         // state.setup();

//         state.pieces = PieceGrid::from(pieces.as_slice());

//         Ok(state)
//     }

// pub fn setup(&mut self) {
//     self.piece_grid = 0;
//     for grid in self.pieces.iter().take(NUM_PIECES) {
//         self.piece_grid |= grid.0;
//     }
// }

// pub fn square_occupied(&self, pos: u8) -> bool {
//     self.piece_grid & (1u64 << pos) > 0
//     // self.pieces
//     //     .iter()
//     //     .map(|x| x.0 & (1u64 << pos) > 0)
//     //     .any(|x| x)
// }

// pub fn get_piece_at_pos(&self, pos: u8) -> Option<Piece> {
//     Some(
//         self.pieces
//             .iter()
//             .enumerate()
//             .find(|x| x.1 .0 & (1u64 << pos) > 0)?
//             .0
//             .into(),
//     )
// }
// }

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct GameState {
    pub piece_grid: PieceGrid,
    pub metadata: Metadata,
    pub player: Player,
    #[serde(default)]
    pub hash: HashType,
}

// fn from_player_vec<'de, D>(deserializer: D) -> Result<[PlayerState; 2], D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let states: BTreeMap<String, PlayerState> = Deserialize::deserialize(deserializer)?;

//     let mut res = [
//         PlayerState::new(Player::White),
//         PlayerState::new(Player::White),
//     ];

//     for (player, state) in states.into_iter() {
//         let player: Player = player.as_str().try_into().map_err(D::Error::custom)?;
//         res[player as usize] = state;
//     }

//     Ok(res)
// }

impl Hash for GameState {
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

    // pub fn new_from_state(
    //     white_player_state: PlayerState,
    //     black_player_state: PlayerState,
    //     en_passant_column: Option<u8>,
    //     player_to_move: Player,
    // ) -> GameState {
    //     GameState {
    //         states: [white_player_state, black_player_state],
    //         en_passant_column,
    //         player: player_to_move,
    //         hash: 0,
    //     }
    // }

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

        // let mut pieces = [Grid::default(); 6];
        // let mut state = Self::default();

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

    pub fn new_from_fen(fen: &str) -> Result<Self, &'static str> {
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

        Ok(Self {
            piece_grid,
            metadata,
            player: Player::try_from(parts[1].chars().nth(0).ok_or("invalid fen")?)?,
            hash: 0,
        })
    }

    pub fn apply_meta_hash(&mut self, zoborist_state: &ZoboristState) {
        // self.hash ^= zoborist_state.castle[self.states[0].meta.can_castle_long as usize]
        //     [self.states[0].meta.can_castle_short as usize]
        //     [self.states[1].meta.can_castle_long as usize]
        //     [self.states[1].meta.can_castle_short as usize];
        // if let Some(en_passant_column) = self.en_passant_column {
        //     self.hash ^= zoborist_state.en_passant[en_passant_column as usize]
        // }
        self.hash ^= self.metadata.get_meta_hash(zoborist_state)
    }

    fn apply_piece_hash(
        &mut self,
        zoborist_state: &ZoboristState,
        square_type: SquareType,
        pos: u8,
    ) {
        // self.hash ^= zoborist_state.pieces[player as usize][piece as usize][pos as usize];
        self.hash ^= square_type.get_piece_hash(pos, zoborist_state)
    }

    pub fn change_player(&mut self, zoborist_state: &ZoboristState) {
        self.hash ^= zoborist_state.player;
        self.player = self.player.opp();
    }

    fn apply_piece_move(
        &mut self,
        zoborist_state: &ZoboristState,
        square_type: SquareType,
        pos: u8,
    ) {
        self.apply_piece_hash(zoborist_state, square_type, pos);
        // let player_state: &mut PlayerState = &mut self.states[player as usize];

        // player_state.pieces[piece as usize].0 ^= 1u64 << pos;
        // player_state.piece_grid ^= 1u64 << pos;
        // self.hash ^= zoborist_state.pieces[player as usize][piece as usize][pos as usize];
        // let square_type_old = self.piece_grid.get_square_type(pos);
        self.piece_grid.apply_square(pos, square_type);
        // let square_type_new = self.piece_grid.get_square_type(pos);
        // if square_type_new.to_raw() == 1{
        //     println!("fail {} {} {} {}", square_type.to_raw(), square_type_old.to_raw(), square_type_new.to_raw(), pos);
        //     panic!();
        // }
    }

    // zoborist hash
    // fn slow_compute_hash(&mut self, zoborist_state: &ZoboristState) {
    //     self.apply_meta_hash(zoborist_state);
    //     for player in [Player::White, Player::Black] {
    //         for piece in [
    //             Piece::Pawn,
    //             Piece::Knight,
    //             Piece::Bishop,
    //             Piece::Rook,
    //             Piece::Queen,
    //             Piece::King,
    //         ] {
    //             for pos in 0..64 {
    //                 if self.get_state(player).pieces[piece as usize].0 & (1 << pos) > 0 {
    //                     self.apply_piece_hash(zoborist_state, player, piece, pos);
    //                 }
    //             }
    //         }
    //     }
    // }

    // pub fn setup(&mut self, zoborist_state: &ZoboristState) {
    //     self.states.iter_mut().for_each(|x| x.setup());
    //     self.slow_compute_hash(zoborist_state);
    // }

    // pub fn get_state(&self, player: Player) -> &PlayerState {
    //     &self.states[player as usize]
    // }

    // fn get_state_mut(&mut self, player: Player) -> &mut PlayerState {
    //     &mut self.states[player as usize]
    // }

    fn modify_state<const APPLY_METADATA_CHANGES: bool>(
        &mut self,
        next_move: Move,
        zoborist_state: &ZoboristState,
    ) {
        if APPLY_METADATA_CHANGES {
            self.metadata.set_en_passant_column(Metadata::NO_EN_PASSANT);
            // self.en_passant_column = None;
        }
        // let offset = if self.player == Player::White { 0 } else { 56 };
        let offset = self.player as u8 * 56;
        match next_move {
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece,
            } => {
                // let square_type_old = self.piece_grid.get_square_type(new_pos);
                // let old_square_type_old = self.piece_grid.get_square_type(prev_pos);
                // if square_type_old.to_raw() == 1 {
                //     println!("old fail old");
                // }
                // if old_square_type_old.to_raw() == 1 {
                //     println!("old fail new");
                // }

                // capture/uncapture the piece if any
                // if let Some(captured_piece) = captured_piece {
                //     self.apply_piece_move(
                //         zoborist_state,
                //         self.player.opp(),
                //         captured_piece,
                //         new_pos,
                //     );
                //     // self.get_state_mut(self.player.opp()).pieces[captured_piece as usize].0 ^=
                //     //     1 << new_pos;
                //     // self.get_state_mut(self.player.opp()).piece_grid ^= 1 << new_pos;
                // }
                self.apply_piece_move(zoborist_state, captured_piece, new_pos);

                // move the current piece
                // self.get_state_mut(self.player).pieces[piece as usize].0 ^=
                //     (1 << prev_pos) | (1 << new_pos);
                // self.get_state_mut(self.player).piece_grid ^= (1 << prev_pos) | (1 << new_pos);
                self.apply_piece_move(zoborist_state, piece, prev_pos);
                self.apply_piece_move(zoborist_state, piece, new_pos);

                if APPLY_METADATA_CHANGES {
                    // let player_state = &mut self.states[self.player as usize];
                    if piece.is_king() || (piece.is_rook() && prev_pos == offset + 7) {
                        // player_state.meta.can_castle_long = false;
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player, false);
                    }
                    if piece.is_king() || (piece.is_rook() && prev_pos == offset) {
                        // player_state.meta.can_castle_short = false;
                        self.metadata
                            .set_can_castle_dynamic::<false>(self.player, true);
                    }

                    // potential en passant next move
                    if piece.is_pawn() && i32::abs(prev_pos as i32 - new_pos as i32) == 16 {
                        self.metadata.set_en_passant_column(prev_pos & 0b111);
                    }
                }
                // let square_type_new = self.piece_grid.get_square_type(new_pos);
                // let old_square_type_new = self.piece_grid.get_square_type(prev_pos);
                // if square_type_new.to_raw() == 1 {
                //     println!("fail old");
                // }
                // if old_square_type_new.to_raw() == 1 {
                //     println!("fail new");
                // }
            }
            Move::Castle { is_short } => {
                // let player_state = &mut self.states[self.player as usize];
                const KING_POS: u8 = 3;
                if APPLY_METADATA_CHANGES {
                    // player_state.meta.can_castle_long = false;
                    // player_state.meta.can_castle_short = false;
                    self.metadata
                        .set_can_castle_dynamic::<false>(self.player, false);
                    self.metadata
                        .set_can_castle_dynamic::<false>(self.player, true);
                }

                let rook_pos = if is_short { 0 } else { 7 };

                let king = SquareType::king(self.player);
                let rook = SquareType::rook(self.player);

                // xor with initial rook/king positions
                self.apply_piece_move(zoborist_state, king, KING_POS + offset);
                self.apply_piece_move(zoborist_state, rook, rook_pos + offset);
                // player_state.piece_grid ^= (1 << (KING_POS + offset)) | (1 << (rook_pos + offset));
                // player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset);
                // player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset);

                // xor with post-castling rook/king positions
                if is_short {
                    // player_state.piece_grid ^=
                    //     (1 << (KING_POS + offset + 2)) | (1 << (rook_pos + offset - 2));
                    // player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset + 2);
                    // player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset - 2);
                    self.apply_piece_move(zoborist_state, king, KING_POS + offset - 2);
                    self.apply_piece_move(zoborist_state, rook, rook_pos + offset + 2);
                } else {
                    // player_state.piece_grid ^=
                    //     (1 << (KING_POS + offset - 2)) | (1 << (rook_pos + offset + 3));
                    // player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset - 2);
                    // player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset + 3);
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
                // capture/uncapture piece if any
                // if let Some(captured_piece) = captured_piece {
                //     self.apply_piece_move(
                //         zoborist_state,
                //         self.player.opp(),
                //         captured_piece,
                //         new_pos,
                //     );
                // }
                self.apply_piece_move(zoborist_state, captured_piece, new_pos);

                // let player_state: &mut PlayerState = &mut self.states[self.player as usize];

                // pawn promotion
                // self.apply_piece_move(zoborist_state, self.player, Piece::Pawn, prev_pos);
                // self.apply_piece_move(zoborist_state, self.player, promoted_to_piece, new_pos);
                self.apply_piece_move(zoborist_state, SquareType::pawn(self.player), prev_pos);
                self.apply_piece_hash(zoborist_state, promoted_to_piece, new_pos);
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
                // self.apply_piece_move(
                //     zoborist_state,
                //     self.player.opp(),
                //     Piece::Pawn,
                //     captured_pawn_pos,
                // );
                self.apply_piece_move(
                    zoborist_state,
                    SquareType::pawn(self.player.opp()),
                    captured_pawn_pos,
                );

                // move the current piece
                // self.apply_piece_move(zoborist_state, self.player, Piece::Pawn, prev_pos);
                // self.apply_piece_move(zoborist_state, self.player, Piece::Pawn, new_pos);
                self.apply_piece_move(zoborist_state, SquareType::pawn(self.player), prev_pos);
                self.apply_piece_move(zoborist_state, SquareType::pawn(self.player), new_pos);

                // let square_type_new = self.piece_grid.get_square_type(new_pos);
                // let old_square_type_new = self.piece_grid.get_square_type(prev_pos);
                // if square_type_new.to_raw() == 1 {
                //     println!(
                //         "fail new {} {}",
                //         square_type_old.to_raw(),
                //         square_type_new.to_raw()
                //     );
                // }
                // if old_square_type_new.to_raw() == 1 {
                //     println!(
                //         "fail old {} {}",
                //         old_square_type_old.to_raw(),
                //         old_square_type_new.to_raw()
                //     );
                // }
            }
        }
    }

    pub fn make_move_raw_parts(
        &mut self,
        prev_pos: u8,
        new_pos: u8,
        promoted_to_piece: Option<Piece>,
    ) -> Result<(), &'static str> {
        // let state = self.get_state(self.player);
        // let opp_state = self.get_state(self.player.opp());
        // let piece = state.get_piece_at_pos(prev_pos).ok_or("invalid from pos")?;
        let piece = self.piece_grid.get_square_type(prev_pos);
        // let captured_piece = opp_state.get_piece_at_pos(new_pos);
        let captured_piece = self.piece_grid.get_square_type(new_pos);
        let next_move = if let Some(promoted_to_piece) = promoted_to_piece {
            let promoted_to_piece = SquareType::create_for_parsing(promoted_to_piece, self.player);
            Move::PawnPromote {
                prev_pos,
                new_pos,
                promoted_to_piece,
                captured_piece,
            }
        } else if piece.is_king() && i32::abs(prev_pos as i32 - new_pos as i32) == 2 {
            Move::Castle {
                is_short: new_pos > prev_pos,
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
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece,
            }
        };

        self.advance_state(next_move, &ZoboristState::STATIC_EMPTY);
        Ok(())
    }

    pub fn advance_state(&mut self, next_move: Move, zoborist_state: &ZoboristState) {
        self.modify_state::<true>(next_move, zoborist_state);
        // switch the player
        self.change_player(zoborist_state);
    }

    pub fn revert_state(&mut self, next_move: Move, zoborist_state: &ZoboristState) {
        // switch the player
        self.change_player(zoborist_state);

        self.modify_state::<false>(next_move, zoborist_state);
    }

    // pub fn score(&self) -> i32 {
    //     self.states[Player::White as usize]
    //         .pieces
    //         .iter()
    //         .enumerate()
    //         .map(|x| PIECE_SCORES[x.0] * x.1 .0.count_ones() as i32)
    //         .sum::<i32>()
    //         - self.states[Player::Black as usize]
    //             .pieces
    //             .iter()
    //             .enumerate()
    //             .map(|x| PIECE_SCORES[x.0] * x.1 .0.count_ones() as i32)
    //             .sum::<i32>()
    // }

    // pub fn is_in_check(&self) -> bool {
    //     let player_state = self.get_state(self.player);
    //     let opp_state = self.get_state(self.player.opp());
    //     let king_pos = (player_state.pieces[Piece::King as usize].0 - 1).count_ones() as u8;
    //     let coord_pos = pos_to_coord(king_pos);

    //     const KNIGHT_DELTAS: [(i8, i8); 8] = [
    //         (-1, -2),
    //         (-1, 2),
    //         (1, -2),
    //         (1, 2),
    //         (-2, -1),
    //         (-2, 1),
    //         (2, -1),
    //         (2, 1),
    //     ];

    //     for (dy, dx) in KNIGHT_DELTAS.into_iter() {
    //         let new_coord = (coord_pos.0 + dy, coord_pos.1 + dx);
    //         if new_coord.0 < 0 || new_coord.0 > 7 || new_coord.1 < 0 || new_coord.1 > 7 {
    //             continue;
    //         }
    //         if opp_state.pieces[Piece::Knight as usize].contains_pos(coord_to_pos(new_coord)) {
    //             return true;
    //         }
    //     }

    //     const BISHOP_DELTAS: [(i8, i8); 4] = [(1, -1), (1, 1), (-1, -1), (-1, 1)];
    //     const ROOK_DELTAS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, -1), (0, 1)];

    //     let helper = |deltas, pieces: [Piece; 2]| {
    //         for (dy, dx) in deltas {
    //             let mut res_coord = util::pos_to_coord(king_pos);
    //             res_coord.0 += dy;
    //             res_coord.1 += dx;
    //             while res_coord.0 >= 0 && res_coord.0 < 8 && res_coord.1 >= 0 && res_coord.1 < 8 {
    //                 let new_pos = util::coord_to_pos(res_coord);

    //                 if pieces
    //                     .iter()
    //                     .map(|x| opp_state.pieces[*x as usize].contains_pos(new_pos))
    //                     .any(|x| x)
    //                 {
    //                     return true;
    //                 }

    //                 if player_state.square_occupied(new_pos) || opp_state.square_occupied(new_pos) {
    //                     break;
    //                 }
    //             }
    //         }
    //         false
    //     };

    //     helper(BISHOP_DELTAS, [Piece::Bishop, Piece::Queen])
    //         || helper(ROOK_DELTAS, [Piece::Rook, Piece::Queen])
    // }
}
