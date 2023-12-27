use crate::{
    grid::Grid, markers::PlayerMarker, movegen, player_to_marker, square_type::SquareType,
    zoborist_state::ZoboristState, GameState, Move,
};

#[derive(Clone, Copy, PartialEq)]
pub struct MoveBufferEntry {
    mov: Move,
    see: Option<i32>,
}

impl MoveBufferEntry {
    pub fn from_move(mov: Move) -> Self {
        Self { mov, see: None }
    }

    #[inline(always)]
    fn get_smallest_attacking_move<P: PlayerMarker>(
        new_pos: u8,
        state: &GameState,
        captured_piece: SquareType,
    ) -> Option<Move> {
        let target_grid = Grid::from_pos(new_pos);

        // pawn
        {
            let all_grid = state.piece_grid.get_pawn_pos::<P>();
            let all_captures = movegen::Pawn::captures::<P>(
                all_grid,
                state.piece_grid.get_opp_player_pieces::<P>(),
            );
            if all_captures & target_grid != Grid::EMPTY {
                for prev_pos in all_grid {
                    let grid = Grid::from_pos(prev_pos);

                    if movegen::Pawn::captures::<P>(
                        grid,
                        state.piece_grid.get_opp_player_pieces::<P>(),
                    ) & target_grid
                        != Grid::EMPTY
                    {
                        return Some(Move::Move {
                            prev_pos,
                            new_pos,
                            piece: SquareType::pawn(P::PLAYER),
                            captured_piece,
                        });
                    }
                }
                unreachable!("should not be reachable: pawn")
            }
        }

        // knight
        {
            let all_grid = state.piece_grid.get_knight_pos::<P>();
            let all_captures =
                movegen::Knight::captures(all_grid, state.piece_grid.get_opp_player_pieces::<P>());
            if all_captures & target_grid != Grid::EMPTY {
                for prev_pos in all_grid {
                    let grid = Grid::from_pos(prev_pos);

                    if movegen::Knight::captures(
                        grid,
                        state.piece_grid.get_opp_player_pieces::<P>(),
                    ) & target_grid
                        != Grid::EMPTY
                    {
                        return Some(Move::Move {
                            prev_pos,
                            new_pos,
                            piece: SquareType::knight(P::PLAYER),
                            captured_piece,
                        });
                    }
                }
                unreachable!("should not be reachable: knight")
            }
        }

        // bishop
        {
            let all_grid = state.piece_grid.get_bishop_pos::<P>();
            let all_captures = movegen::Rays::ray_diagonal_captures(
                all_grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            );
            if all_captures & target_grid != Grid::EMPTY {
                for prev_pos in all_grid {
                    let grid = Grid::from_pos(prev_pos);

                    if movegen::Rays::ray_diagonal_captures(
                        grid,
                        state.piece_grid.get_empty_squares(),
                        state.piece_grid.get_opp_player_pieces::<P>(),
                    ) & target_grid
                        != Grid::EMPTY
                    {
                        return Some(Move::Move {
                            prev_pos,
                            new_pos,
                            piece: SquareType::bishop(P::PLAYER),
                            captured_piece,
                        });
                    }
                }
                unreachable!("should not be reachable: bishop")
            }
        }

        // rook
        {
            let all_grid = state.piece_grid.get_rook_pos::<P>();
            let all_captures = movegen::Rays::ray_horizontal_vertical_captures(
                all_grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            );
            if all_captures & target_grid != Grid::EMPTY {
                for prev_pos in all_grid {
                    let grid = Grid::from_pos(prev_pos);

                    if movegen::Rays::ray_horizontal_vertical_captures(
                        grid,
                        state.piece_grid.get_empty_squares(),
                        state.piece_grid.get_opp_player_pieces::<P>(),
                    ) & target_grid
                        != Grid::EMPTY
                    {
                        return Some(Move::Move {
                            prev_pos,
                            new_pos,
                            piece: SquareType::rook(P::PLAYER),
                            captured_piece,
                        });
                    }
                }
                unreachable!("should not be reachable: rook")
            }
        }

        // queen
        {
            let all_grid = state.piece_grid.get_rook_pos::<P>();
            let all_captures = movegen::Rays::ray_horizontal_vertical_captures(
                all_grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            ) | movegen::Rays::ray_diagonal_captures(
                all_grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            );
            if all_captures & target_grid != Grid::EMPTY {
                for prev_pos in all_grid {
                    let grid = Grid::from_pos(prev_pos);

                    if (movegen::Rays::ray_horizontal_vertical_captures(
                        grid,
                        state.piece_grid.get_empty_squares(),
                        state.piece_grid.get_opp_player_pieces::<P>(),
                    ) | movegen::Rays::ray_diagonal_captures(
                        grid,
                        state.piece_grid.get_empty_squares(),
                        state.piece_grid.get_opp_player_pieces::<P>(),
                    )) & target_grid
                        != Grid::EMPTY
                    {
                        return Some(Move::Move {
                            prev_pos,
                            new_pos,
                            piece: SquareType::queen(P::PLAYER),
                            captured_piece,
                        });
                    }
                }
                unreachable!("should not be reachable: queen")
            }
        }

        // king
        {
            let grid = state.piece_grid.get_king_pos::<P>();
            // the king may be captured during see search
            if grid.num_pieces() > 0 {
                if grid.num_pieces() != 1 {
                    println!("pos: {}", new_pos);
                    state.piece_grid.debug_print();
                    
                }
                let prev_pos = grid.to_pos();
                let grid = Grid::from_pos(prev_pos);

                if movegen::King::captures(grid, state.piece_grid.get_opp_player_pieces::<P>())
                    & target_grid
                    != Grid::EMPTY
                {
                    return Some(Move::Move {
                        prev_pos,
                        new_pos,
                        piece: SquareType::king(P::PLAYER),
                        captured_piece,
                    });
                }
            }
        }

        None
    }

    fn internal_compute_see(pos: u8, state: &mut GameState) -> i32 {
        let mut res = 0;
        let captured_piece = state.piece_grid.get_square_type(pos);
        assert!(!captured_piece.is_empty());

        if let Some(mov) = player_to_marker!(state.player, {
            Self::get_smallest_attacking_move::<P>(pos, state, captured_piece)
        }) {
            state.advance_state_no_metadata_update(mov, &ZoboristState::STATIC_EMPTY);

            res = i32::max(0, captured_piece.value() as i32 - Self::internal_compute_see(pos, state));

            state.revert_state(mov, &ZoboristState::STATIC_EMPTY);
        }

        res
    }

    pub fn get_move(&self) -> Move {
        self.mov
    }

    pub fn get_see_exn(&self) -> i32 {
        self.see.expect("see should exist")
    }

    pub fn has_see(&self) -> bool {
        self.see.is_some()
    }

    pub fn compute_see(&mut self, state: &mut GameState) {
        assert!(self.see.is_none());
        self.see.get_or_insert_with(|| match self.mov {
            Move::Move {
                new_pos,
                captured_piece,
                ..
            }
            | Move::PawnPromote {
                new_pos,
                captured_piece,
                ..
            } => {
                assert!(!captured_piece.is_empty());
                state.advance_state_no_metadata_update(self.mov, &ZoboristState::STATIC_EMPTY);
                let res = captured_piece.value() as i32 - Self::internal_compute_see(new_pos, state);
                // let res = Self::internal_compute_see(new_pos, state);
                state.revert_state(self.mov, &ZoboristState::STATIC_EMPTY);
                // if res < 0{
                // println!("move: {}", self.mov);
                // state.piece_grid.debug_print();
                // println!("see: {}", res);
                // panic!();
                // }
                res
            }
            _ => unreachable!("fail compute see"),
        });
    }
}
