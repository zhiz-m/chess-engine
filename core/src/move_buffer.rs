use crate::{
    config::MAX_CHILDREN_PER_NODE,
    game_data::Metadata,
    grid::Grid,
    markers::*,
    move_buffer_entry::MoveBufferEntry,
    move_orderer::MoveOrderer,
    movegen,
    player::Player,
    square_type::{CompressedSquareType, SquareType},
    types::Move,
    zoborist_state::ZoboristState,
    GameState,
};

#[derive(Clone, PartialEq)]
pub struct MoveBuffer {
    num_moves: usize,
    move_buf: [Option<MoveBufferEntry>; MAX_CHILDREN_PER_NODE],
}

impl Default for MoveBuffer {
    fn default() -> Self {
        Self {
            num_moves: 0,
            move_buf: [None; MAX_CHILDREN_PER_NODE],
        }
    }
}

impl MoveBuffer {
    #[inline(always)]
    fn emit_move(&mut self, next_move: Move) {
        self.move_buf[self.num_moves] = Some(MoveBufferEntry::from_move(next_move));
        self.num_moves += 1;
    }

    #[inline(always)]
    fn get_pawn_moves<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::pawn(P::PLAYER);
        for prev_pos in state.piece_grid.get_pawn_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            // regular moves and captures
            for new_pos in movegen::Pawn::regular_moves_and_captures::<P>(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            )
            .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }

            // double moves
            for new_pos in
                movegen::Pawn::double_moves::<P>(grid, state.piece_grid.get_empty_squares())
                    .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                assert!(captured_piece.is_empty());
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }

            // en passant
            let en_passant = movegen::Pawn::en_passant::<P>(
                grid,
                state.metadata.get_en_passant_column(),
                state.piece_grid.get_empty_squares(),
            );
            if en_passant != Grid::EMPTY
                && state.metadata.get_en_passant_column() != Metadata::NO_EN_PASSANT
            {
                let prev_column = prev_pos & 0b111;
                self.emit_move(Move::EnPassant {
                    prev_column,
                    new_column: state.metadata.get_en_passant_column(),
                })
            }

            // promotions
            for new_pos in movegen::Pawn::promotions::<P>(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            )
            .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);

                for promoted_to_piece in [
                    SquareType::knight(P::PLAYER),
                    SquareType::bishop(P::PLAYER),
                    SquareType::rook(P::PLAYER),
                    SquareType::queen(P::PLAYER),
                ] {
                    self.emit_move(Move::PawnPromote {
                        prev_pos,
                        new_pos,
                        pieces: CompressedSquareType::from_square_types(
                            promoted_to_piece,
                            captured_piece,
                        ),
                    })
                }
            }
        }
    }

    #[inline(always)]
    fn get_pawn_captures<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::pawn(P::PLAYER);
        for prev_pos in state.piece_grid.get_pawn_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            // captures
            for new_pos in
                movegen::Pawn::captures::<P>(grid, state.piece_grid.get_opp_player_pieces::<P>())
                    .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }

            // en passant
            // let en_passant = movegen::Pawn::en_passant::<P>(
            //     grid,
            //     state.metadata.get_en_passant_column(),
            //     state.piece_grid.get_empty_squares(),
            // );
            // if en_passant != Grid::EMPTY
            //     && state.metadata.get_en_passant_column() != Metadata::NO_EN_PASSANT
            // {
            //     let prev_column = prev_pos & 0b111;
            //     self.emit_move(Move::EnPassant {
            //         prev_column,
            //         new_column: state.metadata.get_en_passant_column(),
            //     })
            // }

            // promotions
            // for new_pos in movegen::Pawn::promotion_captures::<P>(
            //     grid,
            //     state.piece_grid.get_opp_player_pieces::<P>(),
            // )
            // .into_iter()
            // {
            //     let captured_piece = state.piece_grid.get_square_type(new_pos);

            //     for promoted_to_piece in [
            //         SquareType::knight(P::PLAYER),
            //         SquareType::bishop(P::PLAYER),
            //         SquareType::rook(P::PLAYER),
            //         SquareType::queen(P::PLAYER),
            //     ] {
            //         self.emit_move(Move::PawnPromote {
            //             prev_pos,
            //             new_pos,
            //             promoted_to_piece,
            //             captured_piece,
            //         })
            //     }
            // }
        }
    }

    #[inline(always)]
    fn get_knight_moves<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::knight(P::PLAYER);
        for prev_pos in state.piece_grid.get_knight_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            for new_pos in
                movegen::Knight::moves(grid, state.piece_grid.get_player_pieces::<P>()).into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    fn get_knight_captures<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::knight(P::PLAYER);
        for prev_pos in state.piece_grid.get_knight_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            for new_pos in
                movegen::Knight::captures(grid, state.piece_grid.get_opp_player_pieces::<P>())
                    .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    fn get_bishop_moves<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::bishop(P::PLAYER);
        for prev_pos in state.piece_grid.get_bishop_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            for new_pos in (movegen::Rays::ray_diagonal_occluded_with_captures_and_non_captures(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            ) & !grid)
                .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    fn get_bishop_captures<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::bishop(P::PLAYER);
        for prev_pos in state.piece_grid.get_bishop_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            for new_pos in (movegen::Rays::ray_diagonal_captures(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            ) & !grid)
                .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    fn get_rook_moves<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::rook(P::PLAYER);
        for prev_pos in state.piece_grid.get_rook_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            for new_pos in
                (movegen::Rays::ray_horizontal_vertical_occluded_with_captures_and_non_captures(
                    grid,
                    state.piece_grid.get_empty_squares(),
                    state.piece_grid.get_opp_player_pieces::<P>(),
                ) & !grid)
                    .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    fn get_rook_captures<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::rook(P::PLAYER);
        for prev_pos in state.piece_grid.get_rook_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            for new_pos in (movegen::Rays::ray_horizontal_vertical_captures(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            ) & !grid)
                .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    fn get_queen_moves<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::queen(P::PLAYER);
        for prev_pos in state.piece_grid.get_queen_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            // diagonal
            for new_pos in (movegen::Rays::ray_diagonal_occluded_with_captures_and_non_captures(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            ) & !grid)
                .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }

            // horizontal
            for new_pos in
                (movegen::Rays::ray_horizontal_vertical_occluded_with_captures_and_non_captures(
                    grid,
                    state.piece_grid.get_empty_squares(),
                    state.piece_grid.get_opp_player_pieces::<P>(),
                ) & !grid)
                    .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(piece, captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    fn get_queen_captures<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::queen(P::PLAYER);
        for prev_pos in state.piece_grid.get_queen_pos::<P>().into_iter() {
            let grid = Grid::from_pos(prev_pos);

            // diagonal
            for new_pos in (movegen::Rays::ray_diagonal_captures(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            ) & !grid)
                .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(
                    piece,
                    captured_piece),
                })
            }

            // horizontal
            for new_pos in (movegen::Rays::ray_horizontal_vertical_captures(
                grid,
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            ) & !grid)
                .into_iter()
            {
                let captured_piece = state.piece_grid.get_square_type(new_pos);
                self.emit_move(Move::Move {
                    prev_pos,
                    new_pos,
                    pieces: CompressedSquareType::from_square_types(
                    piece,
                    captured_piece),
                })
            }
        }
    }

    #[inline(always)]
    /// P is the attacker
    pub fn get_attacked_grid<P: PlayerMarker>(state: &GameState) -> Grid {
        movegen::Pawn::squares_attacked::<P>(state.piece_grid.get_pawn_pos::<P>())
            | movegen::Knight::moves(state.piece_grid.get_knight_pos::<P>(), Grid::EMPTY)
            | movegen::Rays::ray_horizontal_vertical_occluded_with_captures_and_non_captures(
                state.piece_grid.get_rooks_queens::<P>(),
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            )
            | movegen::Rays::ray_diagonal_occluded_with_captures_and_non_captures(
                state.piece_grid.get_bishops_queens::<P>(),
                state.piece_grid.get_empty_squares(),
                state.piece_grid.get_opp_player_pieces::<P>(),
            )
            | movegen::King::regular_moves(
                state.piece_grid.get_king_pos::<P>(),
                state.piece_grid.get_player_pieces::<P>(),
            )
    }

    #[inline(always)]
    /// P is moving player, O is opposite player
    fn get_king_moves<P: PlayerMarker, O: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::king(P::PLAYER);
        let grid = state.piece_grid.get_king_pos::<P>();
        if grid.num_pieces() != 1 {
            state.piece_grid.debug_print();
        }
        let prev_pos = grid.to_pos();

        // regular moves
        for new_pos in movegen::King::regular_moves(grid, state.piece_grid.get_player_pieces::<P>())
        {
            let captured_piece = state.piece_grid.get_square_type(new_pos);
            self.emit_move(Move::Move {
                prev_pos,
                new_pos,pieces: CompressedSquareType::from_square_types(
                piece,
                captured_piece),
            })
        }

        // castling
        if state.metadata.get_can_castle::<P, CastleShortMarker>()
            || state.metadata.get_can_castle::<P, CastleLongMarker>()
        {
            let attacked_grid = Self::get_attacked_grid::<O>(state);
            let occlusion_grid = state.piece_grid.get_all_pieces() & !grid;

            if state.metadata.get_can_castle::<P, CastleShortMarker>()
                && movegen::King::can_castle::<P, CastleShortMarker>(attacked_grid, occlusion_grid)
            {
                self.emit_move(Move::Castle { is_short: true })
            }
            if state.metadata.get_can_castle::<P, CastleLongMarker>()
                && movegen::King::can_castle::<P, CastleLongMarker>(attacked_grid, occlusion_grid)
            {
                self.emit_move(Move::Castle { is_short: false })
            }
        }
    }

    #[inline(always)]
    /// P is moving player, O is opposite player
    fn get_king_captures<P: PlayerMarker>(&mut self, state: &GameState) {
        let piece = SquareType::king(P::PLAYER);
        let grid = state.piece_grid.get_king_pos::<P>();
        let prev_pos = grid.to_pos();

        // regular moves
        for new_pos in movegen::King::captures(grid, state.piece_grid.get_opp_player_pieces::<P>())
        {
            let captured_piece = state.piece_grid.get_square_type(new_pos);
            self.emit_move(Move::Move {
                prev_pos,
                new_pos,pieces: CompressedSquareType::from_square_types(
                piece,
                captured_piece),
            })
        }
    }

    #[inline(always)]
    pub fn get_all_moves(&mut self, state: &GameState) {
        match state.player {
            Player::White => {
                self.get_pawn_moves::<WhiteMarker>(state);
                self.get_knight_moves::<WhiteMarker>(state);
                self.get_bishop_moves::<WhiteMarker>(state);
                self.get_rook_moves::<WhiteMarker>(state);
                self.get_queen_moves::<WhiteMarker>(state);
                self.get_king_moves::<WhiteMarker, BlackMarker>(state);
            }
            Player::Black => {
                self.get_pawn_moves::<BlackMarker>(state);
                self.get_knight_moves::<BlackMarker>(state);
                self.get_bishop_moves::<BlackMarker>(state);
                self.get_rook_moves::<BlackMarker>(state);
                self.get_queen_moves::<BlackMarker>(state);
                self.get_king_moves::<BlackMarker, WhiteMarker>(state);
            }
        }
    }

    #[inline(always)]
    pub fn get_all_captures(&mut self, state: &GameState) {
        match state.player {
            Player::White => {
                self.get_pawn_captures::<WhiteMarker>(state);
                self.get_knight_captures::<WhiteMarker>(state);
                self.get_bishop_captures::<WhiteMarker>(state);
                self.get_rook_captures::<WhiteMarker>(state);
                self.get_queen_captures::<WhiteMarker>(state);
                self.get_king_captures::<WhiteMarker>(state);
            }
            Player::Black => {
                self.get_pawn_captures::<BlackMarker>(state);
                self.get_knight_captures::<BlackMarker>(state);
                self.get_bishop_captures::<BlackMarker>(state);
                self.get_rook_captures::<BlackMarker>(state);
                self.get_queen_captures::<BlackMarker>(state);
                self.get_king_captures::<BlackMarker>(state);
            }
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<Move> {
        if self.num_moves == 0 {
            None
        } else {
            self.num_moves -= 1;
            self.move_buf[self.num_moves].map(|x| x.get_move())
        }
    }

    #[inline(always)]
    pub fn compute_see(&mut self, state: &mut GameState) {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .map(|x| x.as_mut())
            .filter_map(|x| x)
            .for_each(|entry| {
                if let Move::Move { pieces, .. } = entry.get_move() {
                    let (_, captured_piece) = pieces.to_square_types();
                    if !captured_piece.is_empty() {
                        entry.compute_see(state)
                    }
                }
            })
    }

    #[inline(always)]
    pub fn get_next_move(
        &mut self,
        state: &mut GameState,
        last_move_pos: u8,
        depth: usize,
        player: Player,
        transposition_move: Option<Move>,
        move_orderer: &mut MoveOrderer,
    ) -> Option<Move> {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .filter(|x| x.is_some())
            .min_by(|x, y| {
                move_orderer.cmp_move(
                    x.as_ref().unwrap(),
                    y.as_ref().unwrap(),
                    depth,
                    last_move_pos,
                    player,
                    transposition_move,
                )
            })?
            .take()
            .map(|x| x.get_move())
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.num_moves = 0;
    }

    #[inline(always)]
    // TODO: improve
    pub fn is_stalemate(&self) -> bool {
        self.num_moves == 0
    }

    #[inline(always)]
    pub fn get_next_move_capture_king(&mut self) -> Option<Move> {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .filter(|x| x.is_some())
            .find(|x| match x.unwrap().get_move() {
                Move::Move { pieces, .. } | Move::PawnPromote { pieces, .. } => {
                    let (_, captured_piece) = pieces.to_square_types();
                    captured_piece.is_king()
                }
                _ => false,
            })?
            .take()
            .map(|x| x.get_move())
    }

    #[inline(always)]
    pub fn get_next_move_exact_match(&mut self, transposition_move: Option<Move>) -> Option<Move> {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .filter(|x| x.is_some())
            .find(|x| x.map(|x| x.get_move()) == transposition_move)?
            .take()
            .map(|x| x.get_move())
    }

    #[inline(always)]
    pub fn get_next_move_positive_equal_capture(
        &mut self,
        state: &mut GameState,
        last_move_pos: u8,
        depth: usize,
        player: Player,
        transposition_move: Option<Move>,
        move_orderer: &mut MoveOrderer,
    ) -> Option<Move> {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .filter(|x| x.is_some())
            .filter(|x| {
                if x.unwrap().has_see() {
                    x.unwrap().get_see_exn() >= 0
                } else {
                    false
                }
            })
            .min_by(|x, y| {
                move_orderer.cmp_move(
                    x.as_ref().unwrap(),
                    y.as_ref().unwrap(),
                    depth,
                    last_move_pos,
                    player,
                    transposition_move,
                )
            })?
            .take()
            .map(|x| x.get_move())
    }

    #[inline(always)]
    pub fn get_quiescence_move(
        &mut self,
        state: &mut GameState,
        last_move_pos: u8,
        depth: usize,
        player: Player,
        transposition_move: Option<Move>,
        move_orderer: &mut MoveOrderer,
    ) -> Option<Move> {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .filter(|x| x.is_some())
            .filter(|x| match x.unwrap().get_move() {
                Move::Move {
                    new_pos,
                    pieces,
                    ..
                }
                | Move::PawnPromote {
                    new_pos,
                    pieces,
                    ..
                } =>
                // new_pos == last_move_pos || captured_piece.is_king(),
                {
                    x.unwrap().get_see_exn() >= 0
                }
                _ => false,
            })
            .min_by(|x, y| {
                move_orderer.cmp_move(
                    x.as_ref().unwrap(),
                    y.as_ref().unwrap(),
                    depth,
                    last_move_pos,
                    player,
                    transposition_move,
                )
            })?
            .take()
            .map(|x| x.get_move())
    }

    #[inline(always)]
    pub fn get_next_move_killer(
        &mut self,
        depth: usize,
        move_orderer: &mut MoveOrderer,
    ) -> Option<Move> {
        self.move_buf[0..self.num_moves]
            .iter_mut()
            .filter(|x| x.is_some())
            .find(|x| move_orderer.move_is_killer(x.unwrap().get_move(), depth))?
            .take()
            .map(|x| x.get_move())
    }

    // pub fn remove_noncaptures(&mut self) {
    //     self.move_buf[0..self.num_moves]
    //         .iter_mut()
    //         .filter(|x| x.is_some())
    //         .filter(|x| {
    //             if let Move::Move {
    //                 captured_piece,
    //                 new_pos,
    //                 ..
    //             } = x.unwrap()
    //             {
    //                 if !captured_piece.is_empty() {
    //                     return false;
    //                 }
    //             }
    //             return true;
    //         })
    //         .for_each(|x| *x = None)
    // }

    // pub fn count_captures(&self) -> usize {
    //     self.move_buf[0..self.num_moves]
    //         .iter()
    //         .filter_map(|x| *x)
    //         .filter(|x| {
    //             if let Move::Move {
    //                 captured_piece,
    //                 new_pos,
    //                 ..
    //             } = x
    //             {
    //                 if !captured_piece.is_empty() {
    //                     return true;
    //                 }
    //             }
    //             return false;
    //         })
    //         .count()
    // }

    pub fn num_moves(&self) -> usize {
        self.num_moves
    }
}
