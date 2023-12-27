use crate::{
    grid::{Grid, PieceGrid},
    markers::*,
    Player,
};

struct Common;
/// rays include the pieces' position itself.
pub struct Rays;

pub struct Pawn;
pub struct King;
pub struct Queen;
pub struct Knight;

impl Common {
    const REMOVE_LEFTMOST_COLUMN: u64 = 0x7f7f7f7f_7f7f7f7fu64;
    const REMOVE_RIGHTMOST_COLUMN: u64 = 0xfefefefe_fefefefeu64;
    const REMOVE_BOTTOMMOST_ROW: u64 = 0xffffffff_ffffff00u64;
    const REMOVE_TOPMOST_ROW: u64 = 0x00ffffff_ffffffffu64;

    #[inline(always)]
    fn up(grid: Grid) -> Grid {
        grid << 8u8
    }

    #[inline(always)]
    fn down(grid: Grid) -> Grid {
        grid >> 8u8
    }

    // 00000000
    // 00000000
    // 00000000

    #[inline(always)]
    fn left(grid: Grid) -> Grid {
        let grid = grid & Self::REMOVE_LEFTMOST_COLUMN;
        grid << 1u8
    }

    #[inline(always)]
    fn right(grid: Grid) -> Grid {
        let grid = grid & Self::REMOVE_RIGHTMOST_COLUMN;
        grid >> 1u8
    }

    #[inline(always)]
    fn top_left(grid: Grid) -> Grid {
        let grid = grid & Self::REMOVE_LEFTMOST_COLUMN;
        grid << 9u8
    }

    #[inline(always)]
    fn top_right(grid: Grid) -> Grid {
        let grid = grid & Self::REMOVE_RIGHTMOST_COLUMN;
        grid << 7u8
    }

    #[inline(always)]
    fn bottom_right(grid: Grid) -> Grid {
        let grid = grid & Self::REMOVE_RIGHTMOST_COLUMN;
        grid >> 9u8
    }

    #[inline(always)]
    fn bottom_left(grid: Grid) -> Grid {
        let grid = grid & Self::REMOVE_LEFTMOST_COLUMN;
        grid >> 7u8
    }

    // todo: can optimise a few instructions away by making standalone one-square diagonal movements

    // #[inline(always)]
    // pub fn up_occluded(grid: Grid, occlusion_grid: Grid) -> Grid{
    //     Self::up(grid) & !occlusion_grid
    // }

    // #[inline(always)]
    // pub fn down_occluded(grid: Grid, occlusion_grid: Grid) -> Grid{
    //     Self::down(grid) & !occlusion_grid
    // }

    // #[inline(always)]
    // pub fn left_occluded(grid: Grid, occlusion_grid: Grid) -> Grid{
    //     Self::left(grid) & !occlusion_grid
    // }

    // #[inline(always)]
    // pub fn right_occluded(grid: Grid, occlusion_grid: Grid) -> Grid{
    //     Self::right(grid) & !occlusion_grid
    // }
}

impl Rays {
    #[inline(always)]
    fn ray_up_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        // +1
        grid |= (grid << 8u8) & empty_grid;
        empty_grid &= empty_grid << 8u8;
        // +1 +2
        grid |= (grid << 16u8) & empty_grid;
        empty_grid &= empty_grid << 16u8;
        // +1 +2 +4
        grid |= (grid << 32u8) & empty_grid;
        grid
    }

    #[inline(always)]
    fn ray_down_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        // +1
        grid |= (grid >> 8u8) & empty_grid;
        empty_grid &= empty_grid >> 8u8;
        // +1 +2
        grid |= (grid >> 16u8) & empty_grid;
        empty_grid &= empty_grid >> 16u8;
        // +1 +2 +4
        grid |= (grid >> 32u8) & empty_grid;
        grid
    }

    #[inline(always)]
    fn ray_left_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        // remove rightmost column from empty pieces. left shift will never result in
        // wrap-around being an issue.
        empty_grid = empty_grid & Common::REMOVE_RIGHTMOST_COLUMN;

        // +1
        grid |= (grid << 1u8) & empty_grid;
        empty_grid &= empty_grid << 1u8;
        // +1 +2
        grid |= (grid << 2u8) & empty_grid;
        empty_grid &= empty_grid << 2u8;
        // +1 +2 +4
        grid |= (grid << 4u8) & empty_grid;
        grid
    }

    #[inline(always)]
    fn ray_right_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        // remove leftmost column from empty pieces. right shift will never result in
        // wrap-around being an issue.
        empty_grid = empty_grid & Common::REMOVE_LEFTMOST_COLUMN;

        // +1
        grid |= (grid >> 1u8) & empty_grid;
        empty_grid &= empty_grid >> 1u8;
        // +1 +2
        grid |= (grid >> 2u8) & empty_grid;
        empty_grid &= empty_grid >> 2u8;
        // +1 +2 +4
        grid |= (grid >> 4u8) & empty_grid;
        grid
    }

    #[inline(always)]
    fn ray_top_left_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        empty_grid = empty_grid & Common::REMOVE_RIGHTMOST_COLUMN;

        // +1
        grid |= (grid << 9u8) & empty_grid;
        empty_grid &= empty_grid << 9u8;
        // +1 +2
        grid |= (grid << 18u8) & empty_grid;
        empty_grid &= empty_grid << 18u8;
        // +1 +2 +4
        grid |= (grid << 36u8) & empty_grid;
        grid
    }

    #[inline(always)]
    fn ray_top_right_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        empty_grid = empty_grid & Common::REMOVE_LEFTMOST_COLUMN;

        // +1
        grid |= (grid << 7u8) & empty_grid;
        empty_grid &= empty_grid << 7u8;
        // +1 +2
        grid |= (grid << 14u8) & empty_grid;
        empty_grid &= empty_grid << 14u8;
        // +1 +2 +4
        grid |= (grid << 28u8) & empty_grid;
        grid
    }

    #[inline(always)]
    fn ray_bottom_right_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        empty_grid = empty_grid & Common::REMOVE_LEFTMOST_COLUMN;

        // +1
        grid |= (grid >> 9u8) & empty_grid;
        empty_grid &= empty_grid >> 9u8;
        // +1 +2
        grid |= (grid >> 18u8) & empty_grid;
        empty_grid &= empty_grid >> 18u8;
        // +1 +2 +4
        grid |= (grid >> 36u8) & empty_grid;
        grid
    }

    #[inline(always)]
    fn ray_bottom_left_occluded(mut grid: Grid, mut empty_grid: Grid) -> Grid {
        empty_grid = empty_grid & Common::REMOVE_RIGHTMOST_COLUMN;

        // +1
        grid |= (grid >> 7u8) & empty_grid;
        empty_grid &= empty_grid >> 7u8;
        // +1 +2
        grid |= (grid >> 14u8) & empty_grid;
        empty_grid &= empty_grid >> 14u8;
        // +1 +2 +4
        grid |= (grid >> 28u8) & empty_grid;
        grid
    }

    #[inline(always)]
    pub fn ray_horizontal_vertical_occluded_noncaptures(grid: Grid, empty_grid: Grid) -> Grid {
        Self::ray_left_occluded(grid, empty_grid)
            | Self::ray_right_occluded(grid, empty_grid)
            | Self::ray_up_occluded(grid, empty_grid)
            | Self::ray_down_occluded(grid, empty_grid)
    }

    #[inline(always)]
    pub fn ray_horizontal_vertical_captures(
        grid: Grid,
        empty_grid: Grid,
        opposing_pieces: Grid,
    ) -> Grid {
        let left = Self::ray_left_occluded(grid, empty_grid);
        let right = Self::ray_right_occluded(grid, empty_grid);
        let up = Self::ray_up_occluded(grid, empty_grid);
        let down = Self::ray_down_occluded(grid, empty_grid);
        (Common::left(left) | Common::right(right) | Common::up(up) | Common::down(down))
            & opposing_pieces
    }

    #[inline(always)]
    /// includes the original piece positions
    pub fn ray_horizontal_vertical_occluded_with_captures_and_non_captures(
        grid: Grid,
        empty_grid: Grid,
        opposing_pieces: Grid,
    ) -> Grid {
        let left = Self::ray_left_occluded(grid, empty_grid);
        let right = Self::ray_right_occluded(grid, empty_grid);
        let up = Self::ray_up_occluded(grid, empty_grid);
        let down = Self::ray_down_occluded(grid, empty_grid);
        left | right
            | up
            | down
            | ((Common::left(left) | Common::right(right) | Common::up(up) | Common::down(down))
                & opposing_pieces)
    }

    #[inline(always)]
    pub fn ray_diagonal_occluded_noncaptures(grid: Grid, empty_grid: Grid) -> Grid {
        Self::ray_bottom_left_occluded(grid, empty_grid)
            | Self::ray_bottom_right_occluded(grid, empty_grid)
            | Self::ray_top_left_occluded(grid, empty_grid)
            | Self::ray_top_right_occluded(grid, empty_grid)
    }

    #[inline(always)]
    pub fn ray_diagonal_captures(grid: Grid, empty_grid: Grid, opposing_pieces: Grid) -> Grid {
        let bottom_left = Self::ray_bottom_left_occluded(grid, empty_grid);
        let bottom_right = Self::ray_bottom_right_occluded(grid, empty_grid);
        let top_left = Self::ray_top_left_occluded(grid, empty_grid);
        let top_right = Self::ray_top_right_occluded(grid, empty_grid);
        (Common::bottom_left(bottom_left)
            | Common::bottom_right(bottom_right)
            | Common::top_left(top_left)
            | Common::top_right(top_right))
            & opposing_pieces
    }

    #[inline(always)]
    /// includes the original piece positions
    pub fn ray_diagonal_occluded_with_captures_and_non_captures(
        grid: Grid,
        empty_grid: Grid,
        opposing_pieces: Grid,
    ) -> Grid {
        let bottom_left = Self::ray_bottom_left_occluded(grid, empty_grid);
        let bottom_right = Self::ray_bottom_right_occluded(grid, empty_grid);
        let top_left = Self::ray_top_left_occluded(grid, empty_grid);
        let top_right = Self::ray_top_right_occluded(grid, empty_grid);
        bottom_left
            | bottom_right
            | top_left
            | top_right
            | ((Common::bottom_left(bottom_left)
                | Common::bottom_right(bottom_right)
                | Common::top_left(top_left)
                | Common::top_right(top_right))
                & opposing_pieces)
    }
}

impl Knight {
    #[inline(always)]
    pub fn moves(grid: Grid, player_pieces: Grid) -> Grid {
        let up = Common::up(grid);
        let down = Common::down(grid);
        let left = Common::left(grid);
        let right = Common::right(grid);
        (Common::top_left(up)
            | Common::top_right(up)
            | Common::top_left(left)
            | Common::bottom_left(left)
            | Common::bottom_left(down)
            | Common::bottom_right(down)
            | Common::bottom_right(right)
            | Common::top_right(right))
            & !player_pieces
    }
    #[inline(always)]
    pub fn captures(grid: Grid, opposing_pieces: Grid) -> Grid {
        let up = Common::up(grid);
        let down = Common::down(grid);
        let left = Common::left(grid);
        let right = Common::right(grid);
        (Common::top_left(up)
            | Common::top_right(up)
            | Common::top_left(left)
            | Common::bottom_left(left)
            | Common::bottom_left(down)
            | Common::bottom_right(down)
            | Common::bottom_right(right)
            | Common::top_right(right))
            & opposing_pieces
    }
}

impl Pawn {
    const WHITE_PAWN_ROW: Grid = Grid::from_u64(0xff00);
    const BLACK_PAWN_ROW: Grid = Grid::from_u64(0x00ff0000_00000000);
    const RIGHTMOST_COLUMN: Grid = Grid::from_u64(0x01010101_01010101);
    const BLACK_EN_PASSANT_SQUARES: Grid = Grid::from_u64(0xff000000);
    const WHITE_EN_PASSANT_SQUARES: Grid = Grid::from_u64(0xff_00000000);

    #[inline(always)]
    const fn home_pawn_row<P: PlayerMarker>() -> Grid {
        if P::IS_WHITE {
            Self::WHITE_PAWN_ROW
        } else {
            Self::BLACK_PAWN_ROW
        }
    }

    #[inline(always)]
    const fn opposing_home_pawn_row<P: PlayerMarker>() -> Grid {
        if !P::IS_WHITE {
            Self::WHITE_PAWN_ROW
        } else {
            Self::BLACK_PAWN_ROW
        }
    }

    #[inline(always)]
    const fn en_passant_squares<P: PlayerMarker>() -> Grid {
        if P::IS_WHITE {
            Self::WHITE_EN_PASSANT_SQUARES
        } else {
            Self::BLACK_EN_PASSANT_SQUARES
        }
    }

    #[inline(always)]
    /// all possible
    pub fn squares_attacked<P: PlayerMarker>(grid: Grid) -> Grid {
        if P::IS_WHITE {
            Common::top_left(grid) | Common::top_right(grid)
        } else {
            Common::bottom_left(grid) | Common::bottom_right(grid)
        }
    }

    #[inline(always)]
    pub fn regular_moves<P: PlayerMarker>(grid: Grid, empty_grid: Grid) -> Grid {
        let grid = grid & !Self::opposing_home_pawn_row::<P>();
        empty_grid
            & if P::IS_WHITE {
                Common::up(grid)
            } else {
                Common::down(grid)
            }
    }

    #[inline(always)]
    pub fn captures<P: PlayerMarker>(grid: Grid, opposing_pieces: Grid) -> Grid {
        let grid = grid & !Self::opposing_home_pawn_row::<P>();
        Self::squares_attacked::<P>(grid) & opposing_pieces
    }

    #[inline(always)]
    pub fn regular_moves_and_captures<P: PlayerMarker>(
        grid: Grid,
        empty_grid: Grid,
        opposing_pieces: Grid,
    ) -> Grid {
        Self::regular_moves::<P>(grid, empty_grid) | Self::captures::<P>(grid, opposing_pieces)
    }

    #[inline(always)]
    pub fn double_moves<P: PlayerMarker>(grid: Grid, empty_grid: Grid) -> Grid {
        if P::IS_WHITE {
            Common::up(Common::up(grid & Self::home_pawn_row::<P>()) & empty_grid) & empty_grid
        } else {
            Common::down(Common::down(grid & Self::home_pawn_row::<P>()) & empty_grid) & empty_grid
        }
    }

    #[inline(always)]
    pub fn regular_promotions<P: PlayerMarker>(grid: Grid, empty_grid: Grid) -> Grid {
        let grid = grid & Self::opposing_home_pawn_row::<P>();
        empty_grid
            & if P::IS_WHITE {
                Common::up(grid)
            } else {
                Common::down(grid)
            }
    }

    #[inline(always)]
    pub fn promotion_captures<P: PlayerMarker>(grid: Grid, opposing_pieces: Grid) -> Grid {
        let grid = grid & Self::opposing_home_pawn_row::<P>();
        Self::squares_attacked::<P>(grid) & opposing_pieces
    }

    #[inline(always)]
    pub fn promotions<P: PlayerMarker>(grid: Grid, empty_grid: Grid, opposing_pieces: Grid) -> Grid {
        Self::regular_promotions::<P>(grid, empty_grid)
            | Self::promotion_captures::<P>(grid, opposing_pieces)
    }

    #[inline(always)]
    pub fn en_passant<P: PlayerMarker>(
        grid: Grid,
        en_passant_column: u8,
        empty_grid: Grid,
    ) -> Grid {
        let grid = grid & Self::en_passant_squares::<P>();
        match P::PLAYER {
            Player::White => {
                (Common::top_left(grid) | Common::top_right(grid))
                    & (Self::RIGHTMOST_COLUMN << en_passant_column)
                    & empty_grid
            }
            Player::Black => {
                (Common::bottom_left(grid) | Common::bottom_right(grid))
                    & (Self::RIGHTMOST_COLUMN << en_passant_column)
                    & empty_grid
            }
        }
    }
}

impl King {
    #[inline(always)]
    pub fn pieces_attacking<P: PlayerMarker>(
        piece_grid: PieceGrid,
        empty_grid: Grid,
        opposing_pieces: Grid,
    ) -> Grid {
        Rays::ray_diagonal_occluded_with_captures_and_non_captures(
            piece_grid.get_bishops_queens::<P>(),
            empty_grid,
            opposing_pieces,
        ) | Rays::ray_horizontal_vertical_occluded_with_captures_and_non_captures(
            piece_grid.get_rooks_queens::<P>(),
            empty_grid,
            opposing_pieces,
        ) | Knight::moves(piece_grid.get_knight_pos::<P>(), Grid::EMPTY)
            | Pawn::squares_attacked::<P>(piece_grid.get_pawn_pos::<P>())
            | King::regular_moves(piece_grid.get_king_pos::<P>(), Grid::EMPTY)
    }

    #[inline(always)]
    pub fn captures(grid: Grid, opposing_pieces: Grid) -> Grid {
        (Common::up(grid)
            | Common::left(grid)
            | Common::down(grid)
            | Common::right(grid)
            | Common::top_left(grid)
            | Common::top_right(grid)
            | Common::bottom_left(grid)
            | Common::bottom_right(grid))
            & opposing_pieces
    }

    #[inline(always)]
    pub fn regular_moves(grid: Grid, player_pieces: Grid) -> Grid {
        (Common::up(grid)
            | Common::left(grid)
            | Common::down(grid)
            | Common::right(grid)
            | Common::top_left(grid)
            | Common::top_right(grid)
            | Common::bottom_left(grid)
            | Common::bottom_right(grid))
            & !player_pieces
    }

    const WHITE_CASTLE_SHORT_FREE: Grid = Grid::from_u64(0xe);
    const WHITE_CASTLE_LONG_FREE: Grid = Grid::from_u64(0xe << 2);
    const BLACK_CASTLE_SHORT_FREE: Grid = Grid::from_u64(0xe << 56);
    const BLACK_CASTLE_LONG_FREE: Grid = Grid::from_u64(0xe << 58);

    #[inline(always)]
    const fn castle_free_grid<P: PlayerMarker, C: CastleTypeMarker>() -> Grid {
        if C::IS_SHORT {
            if P::IS_WHITE {
                Self::WHITE_CASTLE_SHORT_FREE
            } else {
                Self::BLACK_CASTLE_SHORT_FREE
            }
        } else {
            if P::IS_WHITE {
                Self::WHITE_CASTLE_LONG_FREE
            } else {
                Self::BLACK_CASTLE_LONG_FREE
            }
        }
    }

    #[inline(always)]
    /// occlusion grid must not include the player king.
    pub fn can_castle<P: PlayerMarker, C: CastleTypeMarker>(
        attacked_grid: Grid,
        occlusion_grid: Grid,
    ) -> bool {
        Self::castle_free_grid::<P, C>() & (attacked_grid | occlusion_grid) == Grid::EMPTY
    }
}
