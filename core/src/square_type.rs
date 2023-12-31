use crate::{types_for_io::Piece, player::Player, zoborist_state::ZoboristState, config::HashType};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CompressedSquareType(u8);

impl CompressedSquareType{
    pub fn from_square_types(a: SquareType, b: SquareType) -> Self{
        Self((a.0 << 4) + b.0)
    }

    pub fn to_square_types(self) -> (SquareType, SquareType){
        ((self.0 >> 4).into(), (self.0 & 0b1111).into())
    }
}

#[derive(PartialEq, Eq, Clone,Copy)]
pub struct SquareType(u8);

impl From<u8> for SquareType{
    fn from(value: u8) -> Self {
        Self(value)
    }
}

// after bitshift right 1
// empty: !a & !b & !c = 0
// bishop: !a & b & !c = 2
// knight: !a & b & c  = 3
// rook: a & !b & !c   = 4
// pawn: a & !b & c    = 5
// queen: a & b & !c   = 6
// king: a & b & c     = 7

// 0th bit is color

impl SquareType{
    // only used for ease of parsing the initial state
    pub fn create_for_parsing(piece: Piece, player: Player) -> Self{
        let before_player = match piece{
            Piece::Pawn => 5,
            Piece::Knight => 3,
            Piece::Bishop => 2,
            Piece::Rook => 4,
            Piece::Queen => 6,
            Piece::King => 7,
        };
        let res = Self((before_player << 1) | (player as u8));
        res.check_valid();
        res
    }

    pub fn to_piece_for_io(self) -> Option<Piece>{
        match self.0 >> 1{
            0 => None,
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Knight),
            4 => Some(Piece::Rook),
            5 => Some(Piece::Pawn),
            6 => Some(Piece::Queen),
            7 => Some(Piece::King),
            _ => panic!("invalid square type")
        }
    }

    #[inline(always)]
    pub fn from_parts(a: u8, b: u8, c: u8, d: u8) -> Self{
        assert!(a <= 1 && b <= 1 && c <= 1 && d <= 1);
        let res = Self((a << 3) + (b << 2) + (c << 1) + d);
        res.check_valid();
        res
    }

    #[inline(always)]
    pub fn to_raw(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn get_d(self) -> u8 {
        self.0 & 1
    }    
    #[inline(always)]
    pub fn get_c(self) -> u8 {
        (self.0 >> 1) & 1
    }    
    #[inline(always)]
    pub fn get_b(self) -> u8 {
        (self.0 >> 2) & 1
    }    
    #[inline(always)]
    pub fn get_a(self) -> u8 {
        (self.0 >> 3) & 1
    }    
    #[inline(always)]
    pub fn is_king(self) -> bool {
        self.0 >> 1 == 7
    }
    #[inline(always)]
    pub fn is_rook(self) -> bool {
        self.0 >> 1 == 4
    }
    #[inline(always)]
    pub fn is_pawn(self) -> bool {
        self.0 >> 1 == 5
    }
    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.0 >> 1 == 0
    }
    #[inline(always)]
    pub fn check_valid(self) {
        // cannot be empty + black, or with pieces bits set to 1
        let check = self.0 != 1 && (self.0 >> 1) != 1;
        if !check{
            println!("failing square_type: {}", self.0);
        }
        assert! (self.0 != 1 && (self.0 >> 1) != 1)
    }
    #[inline(always)]
    pub fn get_piece_hash(self, pos: u8, zoborist_state: &ZoboristState) -> HashType{
        zoborist_state.pieces[self.0 as usize][pos as usize]
    }
    #[inline(always)]
    pub fn king(player: Player) -> Self{
        Self((7 << 1) | (player as u8))
    }
    #[inline(always)]
    pub fn rook(player: Player) -> Self{
        Self((4 << 1) | (player as u8))
    }
    #[inline(always)]
    pub fn pawn(player: Player) -> Self{
        Self((5 << 1) | (player as u8))
    }
    #[inline(always)]
    pub fn knight(player: Player) -> Self{
        Self((3 << 1) | (player as u8))
    }
    #[inline(always)]
    pub fn bishop(player: Player) -> Self{
        Self((2 << 1) | (player as u8))
    }
    #[inline(always)]
    pub fn queen(player: Player) -> Self{
        Self((6 << 1) | (player as u8))
    }
    #[inline(always)]
    pub fn value(self) -> u32{
        match self.0 >> 1{
            // we allow 0 here to make piece capture calculations a bit more ergonomic
            0 => 0,
            2 => 3,
            3 => 3,
            4 => 5,
            5 => 1,
            6 => 9,
            7 => 1_000,
            _ => panic!("invalid square type")
        }
    }
}



