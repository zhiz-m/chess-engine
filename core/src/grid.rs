use std::{ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr, Sub,
}, rc::Rc, cell::RefCell};

use serde::Deserialize;

use crate::{markers::*, player::Player, square_type::SquareType, types_for_io::Piece, util};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Deserialize)]
pub struct Grid {
    grid: u64,
}

/// for input only
impl<'a, T, U> From<T> for Grid
where
    U: AsRef<str>,
    T: Iterator<Item = U>,
{
    fn from(value: T) -> Self {
        let mut grid = 0;
        for pos in value {
            grid |= 1u64 << util::canonical_to_pos(pos.as_ref());
        }
        Grid { grid }
    }
}

impl Into<u64> for Grid {
    fn into(self) -> u64 {
        self.grid
    }
}

impl<T: Into<u64>> Sub<T> for Grid {
    type Output = Grid;

    fn sub(self, rhs: T) -> Self::Output {
        Self {
            grid: self.grid - rhs.into(),
        }
    }
}

impl Not for Grid {
    type Output = Grid;

    fn not(self) -> Self {
        Self { grid: !self.grid }
    }
}

impl<T: Into<u64>> BitOr<T> for Grid {
    type Output = Grid;

    fn bitor(self, rhs: T) -> Self::Output {
        Self {
            grid: self.grid | rhs.into(),
        }
    }
}

impl<T: Into<u64>> BitAnd<T> for Grid {
    type Output = Grid;

    fn bitand(self, rhs: T) -> Self::Output {
        Self {
            grid: self.grid & rhs.into(),
        }
    }
}

impl<T: Into<u64>> BitXor<T> for Grid {
    type Output = Grid;

    fn bitxor(self, rhs: T) -> Self::Output {
        Self {
            grid: self.grid ^ rhs.into(),
        }
    }
}

impl<T: Into<u64>> BitOrAssign<T> for Grid {
    fn bitor_assign(&mut self, rhs: T) {
        *self = *self | rhs;
    }
}

impl<T: Into<u64>> BitAndAssign<T> for Grid {
    fn bitand_assign(&mut self, rhs: T) {
        *self = *self & rhs;
    }
}

impl<T: Into<u64>> BitXorAssign<T> for Grid {
    fn bitxor_assign(&mut self, rhs: T) {
        *self = *self ^ rhs
    }
}

impl<T: Into<u64>> Shl<T> for Grid {
    type Output = Grid;

    fn shl(self, rhs: T) -> Self::Output {
        Self {
            grid: self.grid << rhs.into(),
        }
    }
}

impl<T: Into<u64>> Shr<T> for Grid {
    type Output = Grid;

    fn shr(self, rhs: T) -> Self::Output {
        Self {
            grid: self.grid >> rhs.into(),
        }
    }
}

impl Grid {
    // pub fn contains_pos(&self, pos: u8) -> bool {
    //     self.grid & (1 << pos) > 0
    // }
    pub fn all_squares_occupied(self, grid: Grid) -> bool {
        (self & grid) == grid
    }

    pub fn to_bool(self) -> bool {
        self.grid != 0
    }

    pub fn num_pieces(self) -> u32 {
        self.grid.count_ones()
    }

    pub const fn from_u64(grid: u64) -> Self {
        Self { grid }
    }

    // self must only contain one pos
    pub fn to_pos(self) -> u8 {
        assert!(self.num_pieces() == 1);
        (self.grid - 1).count_ones() as u8
    }

    pub fn from_pos(pos: u8) -> Self {
        Self { grid: 1 << pos }
    }

    // const ONE: Self = Grid { grid: 1 };
    pub const EMPTY: Self = Grid { grid: 0 };
}

impl IntoIterator for Grid {
    type Item = u8;

    type IntoIter = GridIterator;

    fn into_iter(self) -> Self::IntoIter {
        GridIterator { grid: self }
    }
}

pub struct GridIterator {
    grid: Grid,
}

impl Iterator for GridIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.grid == Grid::EMPTY {
            None
        } else {
            let res = self.grid.grid.trailing_zeros() as u8;
            self.grid = self.grid & (self.grid - 1u8);
            Some(res)
        }
    }
}

#[derive(Clone, Deserialize, Default, PartialEq, Eq)]
pub struct PieceGrid {
    #[serde(default)]
    a: Grid, // pieces that can move horizontally/vertically
    #[serde(default)]
    b: Grid, // pieces that can move diagonally
    #[serde(default)]
    c: Grid, // pieces with special movement
    #[serde(default)]
    d: Grid, // after bitshift right 1
             // empty: !a & !b & !c = 0
             // bishop: !a & b & !c = 2
             // knight: !a & b & c  = 3
             // rook: a & !b & !c   = 4
             // pawn: a & !b & c    = 5
             // queen: a & b & !c   = 6
             // king: a & b & c     = 7
}

impl PieceGrid {
    pub fn all_squares_occupied(&self, grid: Grid) -> bool {
        (self.a | self.b | self.c) & grid == grid
    }

    pub fn get_square_type(&self, pos: u8) -> SquareType {
        // assert!(grid.num_pieces() == 1);
        // let pos = (grid.grid - 1).count_ones() as u8;
        let a = (self.a >> pos).grid & 1;
        let b = (self.b >> pos).grid & 1;
        let c = (self.c >> pos).grid & 1;
        let d = (self.d >> pos).grid & 1;

        SquareType::from_parts(a as u8, b as u8, c as u8, d as u8)
    }

    fn get_d_bits<P: PlayerMarker>(&self) -> Grid {
        let bits = if P::IS_WHITE { u64::MAX } else { 0 }; // ones for white, zero for black
        Grid {
            grid: (bits ^ self.d.grid),
        }
    }

    fn get_opp_d_bits<P: PlayerMarker>(&self) -> Grid {
        let bits = if !P::IS_WHITE { u64::MAX } else { 0 }; // ones for white, zero for black
        Grid {
            grid: (bits ^ self.d.grid),
        }
    }

    pub fn get_pawn_pos<P: PlayerMarker>(&self) -> Grid {
        self.a & !self.b & self.c & self.get_d_bits::<P>()
    }

    pub fn get_knight_pos<P: PlayerMarker>(&self) -> Grid {
        !self.a & self.b & self.c & self.get_d_bits::<P>()
    }

    pub fn get_bishop_pos<P: PlayerMarker>(&self) -> Grid {
        !self.a & self.b & !self.c & self.get_d_bits::<P>()
    }

    pub fn get_rook_pos<P: PlayerMarker>(&self) -> Grid {
        self.a & !self.b & !self.c & self.get_d_bits::<P>()
    }

    pub fn get_queen_pos<P: PlayerMarker>(&self) -> Grid {
        self.a & self.b & !self.c & self.get_d_bits::<P>()
    }

    pub fn get_king_pos<P: PlayerMarker>(&self) -> Grid {
        self.a & self.b & self.c & self.get_d_bits::<P>()
    }

    pub fn get_rooks_queens<P: PlayerMarker>(&self) -> Grid {
        self.a & !self.c & self.get_d_bits::<P>()
    }

    pub fn get_bishops_queens<P: PlayerMarker>(&self) -> Grid {
        self.b & !self.c & self.get_d_bits::<P>()
    }
    pub fn get_empty_squares(&self) -> Grid {
        !(self.a | self.b | self.c)
    }

    pub fn get_player_pieces<P: PlayerMarker>(&self) -> Grid {
        (self.a | self.b | self.c) & self.get_d_bits::<P>()
    }

    pub fn get_opp_player_pieces<P: PlayerMarker>(&self) -> Grid {
        (self.a | self.b | self.c) & self.get_opp_d_bits::<P>()
    }

    pub fn get_all_pieces(&self) -> Grid {
        self.a | self.b | self.c
    }

    pub fn get_squares_of_type(&self, square_type: SquareType) -> Grid {
        let a = u64::MAX + square_type.get_a() as u64;
        let b = u64::MAX + square_type.get_b() as u64;
        let c = u64::MAX + square_type.get_c() as u64;
        let d = u64::MAX + square_type.get_d() as u64;
        (self.a ^ a) & (self.b ^ b) & (self.c ^ c) & (self.d ^ d)
    }

    // note: apply hash elsewhere
    pub fn apply_square(&mut self, pos: u8, square_type: SquareType) {
        // square_type.check_valid();
        self.a ^= Grid {
            grid: (square_type.get_a() as u64) << pos,
        };
        self.b ^= Grid {
            grid: (square_type.get_b() as u64) << pos,
        };
        self.c ^= Grid {
            grid: (square_type.get_c() as u64) << pos,
        };
        self.d ^= Grid {
            grid: (square_type.get_d() as u64) << pos,
        };
        // self.get_square_type(pos);
    }

    pub fn from_piece_vec<T: AsRef<str>>(white_pieces: &[Vec<T>], black_pieces: &[Vec<T>]) -> Self {
        let res = Rc::new(RefCell::new(Self::default()));
        let iter_fn = |player: Player| {
            let res = res.clone();
            move |(piece_type, coords): (usize, &Vec<T>)| {
                let piece = Piece::from(piece_type);
                let grid = Grid::from(coords.iter());
                let square_type = SquareType::create_for_parsing(piece, player);
                grid.into_iter()
                    .for_each(|pos| res.borrow_mut().apply_square(pos, square_type));
            }
        };
        white_pieces
            .into_iter()
            .enumerate()
            .for_each(iter_fn(Player::White));
        black_pieces
            .into_iter()
            .enumerate()
            .for_each(iter_fn(Player::Black));
        res.take()
    }
}

// impl From<&[Grid]> for PieceGrid {
//     fn from(pieces: &[Grid]) -> PieceGrid {
//         let (mut a, mut b, mut c): (Grid, Grid, Grid) = Default::default();
//         a |= pieces[Piece::Pawn as usize]
//             | pieces[Piece::Rook as usize]
//             | pieces[Piece::Queen as usize]
//             | pieces[Piece::King as usize];

//         b |= pieces[Piece::Knight as usize]
//             | pieces[Piece::Bishop as usize]
//             | pieces[Piece::Queen as usize]
//             | pieces[Piece::King as usize];
//         c |= pieces[Piece::Knight as usize]
//             | pieces[Piece::Pawn as usize]
//             | pieces[Piece::King as usize];
//         Self { a, b, c }
//     }
// }
