use std::fmt::Display;

use crate::{square_type::SquareType, types_for_io::Piece, util};

// #[derive(Clone, Copy, PartialEq, Debug, Eq)]
// pawn: a & !b & c
// knight: !a & b & c
// bishop: !a & b & c
// rook: a & !b & !c
// queen: a & b & !c
// king: a & b & c
// pub enum Piece {
//     Pawn = 0,
//     Knight = 1,
//     Bishop = 2,
//     Rook = 3,
//     Queen = 4,
//     King = 5,
// }

// impl Piece {
//     pub fn value(self) -> i32 {
//         // [1, 3, 3, 5, 9, 1_000_000_000]
//         PIECE_SCORES[self as usize]
//     }
// }

// impl Display for Piece {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(match self {
//             Piece::Pawn => "pawn",
//             Piece::Knight => "knight",
//             Piece::Bishop => "bishop",
//             Piece::Rook => "rook",
//             Piece::Queen => "queen",
//             Piece::King => "king",
//         })
//     }
// }

// impl From<usize> for Piece {
//     fn from(value: usize) -> Self {
//         match value {
//             0 => Piece::Pawn,
//             1 => Piece::Knight,
//             2 => Piece::Bishop,
//             3 => Piece::Rook,
//             4 => Piece::Queen,
//             5 => Piece::King,
//             _ => unreachable!(),
//         }
//     }
// }

// impl TryFrom<char> for Piece {
//     type Error = &'static str;

//     fn try_from(value: char) -> Result<Self, Self::Error> {
//         Ok(match value.to_ascii_lowercase() {
//             'p' => Piece::Pawn,
//             'n' => Piece::Knight,
//             'b' => Piece::Bishop,
//             'r' => Piece::Rook,
//             'q' => Piece::Queen,
//             'k' => Piece::King,
//             _ => return Err("invalid piece char"),
//         })
//     }
// }

// impl TryFrom<&str> for Piece {
//     type Error = &'static str;

//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         Ok(match value.to_lowercase().as_str() {
//             "pawn" => Piece::Pawn,
//             "knight" => Piece::Knight,
//             "bishop" => Piece::Bishop,
//             "rook" => Piece::Rook,
//             "queen" => Piece::Queen,
//             "king" => Piece::King,
//             _ => return Err("invalid piece string"),
//         })
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Move {
        prev_pos: u8,
        new_pos: u8,
        piece: SquareType,
        captured_piece: SquareType,
    },
    Castle {
        is_short: bool,
    },
    PawnPromote {
        prev_pos: u8,
        new_pos: u8,
        promoted_to_piece: SquareType,
        captured_piece: SquareType,
    },
    EnPassant {
        prev_column: u8,
        new_column: u8,
    },
}

impl Move {
    // pub fn get_cmp_key(self, last_move_pos: u8, killer_entry: KillerEntry) -> (u8, i32) {
    //     match self {
    //         Move::Move {
    //             new_pos,
    //             piece,
    //             captured_piece,
    //             ..
    //         } => {
    //             if let Some(captured_piece) = captured_piece {
    //                 if captured_piece == Piece::King {
    //                     (0, 0)
    //                 }
    //                 // else if piece.value() > captured_piece.value(){
    //                 //     (5,0)
    //                 // }
    //                 else if new_pos == last_move_pos {
    //                     (1, piece.value() - captured_piece.value())
    //                 } else if piece.value() < captured_piece.value() {
    //                     (3, piece.value() - captured_piece.value())
    //                 }
    //                 // captures worth the same material are slightly preferred over non-killer silent moves
    //                 else {
    //                     (4, piece.value() - captured_piece.value() - 1)
    //                 }
    //             } else if killer_entry.contains(self) {
    //                 // println!("killer move");
    //                 (4, -2)
    //             } else {
    //                 (4, 0)
    //             }
    //         }
    //         Move::Castle { .. } => (4, 0),
    //         Move::PawnPromote {
    //             promoted_to_piece, ..
    //         } => (2, -promoted_to_piece.value()),
    //         Move::EnPassant { .. } => (3, 1),
    //     }
    // }

    // pub fn is_capture(self) -> bool {
    //     match self {
    //         Move::Move {
    //             captured_piece: Some(_),
    //             ..
    //         } => true,
    //         Move::PawnPromote {
    //             captured_piece: Some(_),
    //             ..
    //         } => true,
    //         _ => false,
    //     }
    // }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match *self {
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece,
            } => {
                let piece = piece.to_piece_for_io().expect("cannot be empty");
                let captured_piece = captured_piece.to_piece_for_io();
                match captured_piece {
                    Some(captured_piece) => format!(
                        "{} {} to {}, capturing {}",
                        piece,
                        util::coord_to_canonical(util::pos_to_coord(prev_pos.try_into().unwrap())),
                        util::coord_to_canonical(util::pos_to_coord(new_pos.try_into().unwrap())),
                        captured_piece
                    ),
                    None => format!(
                        "{} {} to {}",
                        piece,
                        util::coord_to_canonical(util::pos_to_coord(prev_pos.try_into().unwrap())),
                        util::coord_to_canonical(util::pos_to_coord(new_pos.try_into().unwrap()))
                    ),
                }
            }
            Move::Castle { is_short: true } => "castle short".into(),
            Move::Castle { is_short: false } => "castle long".into(),
            Move::PawnPromote {
                prev_pos,
                new_pos,
                promoted_to_piece,
                captured_piece,
            } => {
                let captured_piece = captured_piece.to_piece_for_io();
                let promoted_to_piece =
                    promoted_to_piece.to_piece_for_io().expect("cannot be empty");
                match captured_piece {
                    Some(captured_piece) => format!(
                        "{} {} to {}, capturing {}, promoting to {}",
                        Piece::Pawn,
                        util::coord_to_canonical(util::pos_to_coord(prev_pos.try_into().unwrap())),
                        util::coord_to_canonical(util::pos_to_coord(new_pos.try_into().unwrap())),
                        captured_piece,
                        promoted_to_piece
                    ),
                    None => format!(
                        "{} {} to {}, promoting to {}",
                        Piece::Pawn,
                        util::coord_to_canonical(util::pos_to_coord(prev_pos.try_into().unwrap())),
                        util::coord_to_canonical(util::pos_to_coord(new_pos.try_into().unwrap())),
                        promoted_to_piece
                    ),
                }
            }
            Move::EnPassant {
                prev_column,
                new_column,
            } => {
                format!(
                    "En passant from column {} to column {}",
                    prev_column, new_column
                )
            }
        };
        f.write_str(&result)
    }
}

#[derive(PartialEq, Eq)]
pub struct ValueMovePair(pub i32, pub Move);

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
