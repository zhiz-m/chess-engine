use std::fmt::Display;

/// for ease of parsing only. not used in internal engine

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
// empty: !a & !b & !c = 0
// bishop: !a & b & !c = 2
// knight: !a & b & c  = 3
// rook: a & !b & !c   = 4
// pawn: a & !b & c    = 5
// queen: a & b & !c   = 6
// king: a & b & c     = 7

// 0th bit is color
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

// impl Piece {
//     pub fn value(self) -> i32 {
//         // [1, 3, 3, 5, 9, 1_000_000_000]
//         PIECE_SCORES[self as usize]
//     }
// }

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

impl TryFrom<char> for Piece {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value.to_ascii_lowercase() {
            'p' => Piece::Pawn,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'r' => Piece::Rook,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => return Err("invalid piece char"),
        })
    }
}

impl TryFrom<&str> for Piece {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "pawn" => Piece::Pawn,
            "knight" => Piece::Knight,
            "bishop" => Piece::Bishop,
            "rook" => Piece::Rook,
            "queen" => Piece::Queen,
            "king" => Piece::King,
            _ => return Err("invalid piece string"),
        })
    }
}
