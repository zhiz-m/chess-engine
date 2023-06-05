const NUM_PIECES: usize = 6;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    fn opp(self) -> Player {
        if self == Player::White {
            Player::Black
        } else {
            Player::White
        }
    }
}

const PIECE_SCORES: [i32; 6] = [1, 3, 3, 5, 9, 1_000_000_000];

#[derive(Clone, Copy, PartialEq)]
enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece{
    fn value(self) -> i32{
        // [1, 3, 3, 5, 9, 1_000_000_000]
        PIECE_SCORES[self as usize]
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

#[derive(Clone, Debug)]
struct Grid(u64);

impl Grid {}

impl<'a, T> From<T> for Grid
where
    T: IntoIterator<Item = &'a str>,
{
    fn from(value: T) -> Self {
        let mut grid = 0;
        for pos in value {
            grid |= 1u64 << canonical_to_grid(pos);
        }
        Grid(grid)
    }
}

fn canonical_to_grid(xy: &str) -> u8 {
    // println!("item: {}", (xy.as_bytes()[1] << 3) as u8 + xy.as_bytes()[0] as u8 - 'a' as u8);
    ((xy.as_bytes()[1] - b'1') << 3) + xy.as_bytes()[0] - b'a'
}


// cheap, copyable player state
#[derive(Clone, Copy)]
pub struct PlayerMetadata{
    can_castle_short: bool,
    can_castle_long: bool,
}

impl PlayerMetadata{
    pub fn new() -> PlayerMetadata{
        PlayerMetadata { can_castle_short: true, can_castle_long: true }
    }
}

pub struct PlayerState {
    pieces: [Grid; NUM_PIECES],
    piece_grid: u64,
    meta: PlayerMetadata
}

impl PlayerState {
    pub fn new(player: Player) -> PlayerState {
        PlayerState {
            pieces: match player {
                Player::White => [
                    vec!["a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2"].into(),
                    vec!["b1", "g1"].into(),
                    vec!["c1", "f1"].into(),
                    vec!["a1", "h1"].into(),
                    vec!["d1"].into(),
                    vec!["e1"].into(),
                ],
                Player::Black => [
                    vec!["a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7"].into(),
                    vec!["b8", "g8"].into(),
                    vec!["c8", "f8"].into(),
                    vec!["a8", "h8"].into(),
                    vec!["d8"].into(),
                    vec!["e8"].into(),
                ],
            },
            piece_grid: 0xffff00000000ffff,
            meta: PlayerMetadata::new()
        }
    }

    pub fn new_from_state(pos: [Vec<&str>; 6]) -> PlayerState {
        let pieces: [Grid; 6] = pos.into_iter().map(Into::<Grid>::into).collect::<Vec<Grid>>().try_into().unwrap();
        let mut piece_grid = 0;
        for grid in pieces.iter().take(NUM_PIECES){
            for j in 0..64{
                if (1u64 << j) & grid.0 > 0{
                    piece_grid |= 1u64 << j;
                }
            }
        }
        PlayerState { pieces, piece_grid, meta: PlayerMetadata::new() }
    } 

    fn square_occupied(&self, pos: u8) -> bool {
        self.pieces
            .iter()
            .map(|x| x.0 & (1u64 << pos) > 0)
            .any(|x| x)
        /*for i in 0..NUM_PIECES{
            if self.pieces[i].0 & (1u64 << pos) > 0 {
                return false;
            }
        }
        true*/
    }

    fn get_piece_at_pos(&self, pos: u8) -> Option<Piece> {
        Some(
            self.pieces
                .iter()
                .enumerate()
                .find(|x| x.1 .0 & (1u64 << pos) > 0)?
                .0
                .into(),
        )
    }
}

pub struct GameState {
    states: [PlayerState; 2],
    player: Player,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            states: [
                PlayerState::new(Player::White),
                PlayerState::new(Player::Black),
            ],
            player: Player::White,
        }
    }

    pub fn new_from_state(white_player_state: PlayerState, black_player_state: PlayerState, player_to_move: Player) -> GameState{
        
        GameState{
        states: [white_player_state, black_player_state],
        player: player_to_move,
            }    }

    fn get_state(&self, player: Player) -> &PlayerState {
        &self.states[player as usize]
    }

    fn get_state_mut(&mut self, player: Player) -> &mut PlayerState {
        &mut self.states[player as usize]
    }

    fn modify_state<const APPLY_METADATA_CHANGES: bool>(&mut self, val: Move){
        let offset = if self.player == Player::White {0} else {56};
        match val{
            Move::Move { prev_pos, new_pos, piece, captured_piece } => {
                // capture the piece if any
                if let Some(captured_piece) = captured_piece{
                    self.get_state_mut(self.player.opp()).pieces[captured_piece as usize]
                        .0 ^= 1 << new_pos;
                    self.get_state_mut(self.player.opp()).piece_grid ^= 1 << new_pos;
                }

                // move the current piece
                self.get_state_mut(self.player).pieces[piece as usize].0 ^=
                (1 << prev_pos) | (1 << new_pos);
                self.get_state_mut(self.player).piece_grid ^=
                (1 << prev_pos) | (1 << new_pos);

                if APPLY_METADATA_CHANGES{
                    let player_state = &mut self.states[self.player as usize];
                    if piece == Piece::King || (piece == Piece::Rook && prev_pos == offset){
                        player_state.meta.can_castle_long = false;
                    }
                    if piece == Piece::King || (piece == Piece::Rook && prev_pos == 7 + offset) {
                        player_state.meta.can_castle_short = false;
                    }
                }
            },
            Move::Castle { is_short } => {
                let player_state = &mut self.states[self.player as usize];
                const KING_POS: u8 = 4;
                if APPLY_METADATA_CHANGES{
                    player_state.meta.can_castle_long = false;
                    player_state.meta.can_castle_short = false;
                }
                
                let rook_pos = if is_short{7} else {0};

                // xor with initial rook/king positions
                player_state.piece_grid ^= (1 << (KING_POS + offset)) | (1 << (rook_pos + offset));
                player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset);
                player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset);

                // xor with post-castling rook/king positions
                if is_short{
                    player_state.piece_grid ^= (1 << (KING_POS + offset + 2)) | (1 << (rook_pos + offset - 2));
                    player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset + 2);
                    player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset - 2);
                }
                else{
                    player_state.piece_grid ^= (1 << (KING_POS + offset - 2)) | (1 << (rook_pos + offset + 3));
                    player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset - 2);
                    player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset + 3);
                }
            },
        }
    }

    fn advance_state(&mut self, val: Move){
        self.modify_state::<true>(val);
        // switch the player
        self.player = self.player.opp();
    }

    fn revert_state(&mut self, val: Move){
        // switch the player
        self.player = self.player.opp();

        self.modify_state::<false>(val);
    }

    pub fn score(&self) -> i32 {
        self.states[Player::White as usize]
            .pieces
            .iter()
            .enumerate()
            .map(|x| PIECE_SCORES[x.0] * x.1 .0.count_ones() as i32)
            .sum::<i32>()
            - self.states[Player::Black as usize]
                .pieces
                .iter()
                .enumerate()
                .map(|x| PIECE_SCORES[x.0] * x.1 .0.count_ones() as i32)
                .sum::<i32>()
    }
}

// row number then col number
fn pos_to_coord(pos: u8) -> (i8, i8) {
    // println!("a {} {} {}", pos, (pos as i8) << 3, pos as i8 & 0b111);
    ((pos >> 3) as i8, (pos & 0b111) as i8)
}

fn coord_to_pos(coord: (i8, i8)) -> u8 {
    // println!("b {} {}", coord.0, coord.1);
    ((coord.0 as u8) << 3) + coord.1 as u8
}

#[derive(Clone, Copy)]
enum Move{
    Move{prev_pos: u8, new_pos: u8, piece: Piece, captured_piece: Option<Piece>},
    // Capture{prev_pos: u8, new_pos: u8, piece: Piece, captured_piece: Piece},
    Castle{is_short: bool}
}

impl Move{
    fn get_cmp_key(self, last_move_pos: u8) -> (u8, i32){
        match self{
            Move::Move { new_pos, piece, captured_piece, ..} => {
                if let Some(captured_piece) = captured_piece{
                    if new_pos == last_move_pos{
                        (0, piece.value() - captured_piece.value())
                    }
                    else{
                        (1, piece.value() - captured_piece.value())
                    }
                }
                else{
                    (2,0)
                }
            },
            Move::Castle { .. } => (2,0),
        }
    }
}

pub struct ChessEngine<const MAX_DEPTH_SOFT: usize, const MAX_DEPTH_CAPTURE: usize, const MAX_DEPTH_HARD: usize> {
    // state: GameState,
    move_buf: [Vec<Move>; MAX_DEPTH_HARD],
    opp_move_grid: u64,
    nodes_explored: usize,
}

impl<const MAX_DEPTH_SOFT: usize, const MAX_DEPTH_CAPTURE: usize, const MAX_DEPTH_HARD: usize> Default for ChessEngine<MAX_DEPTH_SOFT, MAX_DEPTH_CAPTURE, MAX_DEPTH_HARD> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_DEPTH_SOFT: usize, const MAX_DEPTH_CAPTURE: usize, const MAX_DEPTH_HARD: usize> ChessEngine<MAX_DEPTH_SOFT, MAX_DEPTH_CAPTURE, MAX_DEPTH_HARD> {
    pub fn new() -> Self {
        if MAX_DEPTH_SOFT > MAX_DEPTH_CAPTURE || MAX_DEPTH_CAPTURE > MAX_DEPTH_HARD{
            panic!("invalid depth parameters");
        }
        const BUF: Vec<Move> = vec![];
        ChessEngine {
            // state: GameState::new(),
            opp_move_grid: 0,
            move_buf: [BUF; MAX_DEPTH_HARD],
            nodes_explored: 0,
        }
    }

    // TYPE: whether to save it to teh move buffer or to put i
    fn emit_move<const TYPE: bool>(
        &mut self,
        val: Move,
        depth: usize,
    ) {
        if TYPE {
            self.move_buf[depth].push(val);
        } else if let Move::Move { new_pos, .. } = val {
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
        depth: usize,
    ) {
        for (dy, dx) in deltas {
            let mut res_coord = pos_to_coord(prev_pos);
            res_coord.0 += dy;
            res_coord.1 += dx;
            while res_coord.0 >= 0 && res_coord.0 < 8 && res_coord.1 >= 0 && res_coord.1 < 8 {   
                let new_pos = coord_to_pos(res_coord);
                if state.get_state(player).square_occupied(new_pos) {
                    break;
                }
                // self.emit_move::<TYPE>(prev_pos, new_pos, piece, MoveType::MoveCapture, depth);
                // self.move_buf.push((pos, res_pos, MoveType::MoveCapture));
                self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos, piece, captured_piece: state.get_state(player.opp()).get_piece_at_pos(new_pos) }, depth);
                res_coord.0 += dy;
                res_coord.1 += dx;
            }
        }
    }

    // todo: enpassant, double move
    fn get_move_by_piece<const TYPE: bool>(
        &mut self,
        state: &GameState,
        piece: Piece,
        prev_pos: u8,
        player: Player,
        depth: usize,
    ) {
        match piece {
            Piece::Pawn => {
                if player == Player::White {
                    if !state.get_state(player.opp()).square_occupied(prev_pos + 8) {
                        self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos + 8, piece, captured_piece: None }, depth);
                        if (prev_pos >> 3) == 1 && !state.get_state(player.opp()).square_occupied(prev_pos + 16) {
                            self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos + 16, piece, captured_piece: None }, depth);
                        }
                    }

                    if (prev_pos & 0b111) > 0
                    {
                        if let Some(captured_piece) = state.get_state(player.opp()).get_piece_at_pos(prev_pos + 7){
                            self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos+7, piece, captured_piece: Some(captured_piece) }, depth);
                        }
                    }
                    if (prev_pos & 0b111) < 7
                    {
                        if let Some(captured_piece) = state.get_state(player.opp()).get_piece_at_pos(prev_pos + 9){
                            self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos + 9, piece, captured_piece: Some(captured_piece) }, depth);
                        }
                    }
                } else {
                    if !state.get_state(player.opp()).square_occupied(prev_pos - 8) {
                        self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos - 8, piece, captured_piece: None }, depth);
                        if (prev_pos >> 3) == 6 && !state.get_state(player.opp()).square_occupied(prev_pos - 16) {
                            self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos - 16, piece, captured_piece: None }, depth);
                        }
                    }
                    if (prev_pos & 0b111) > 0
                    {
                        if let Some(captured_piece) = state.get_state(player.opp()).get_piece_at_pos(prev_pos - 9){
                            self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos - 9, piece, captured_piece: Some(captured_piece) }, depth);
                        }
                    }
                    if (prev_pos & 0b111) < 7
                    {
                        if let Some(captured_piece) = state.get_state(player.opp()).get_piece_at_pos(prev_pos - 7){
                            self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos: prev_pos - 7, piece, captured_piece: Some(captured_piece) }, depth);
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
                let coord = pos_to_coord(prev_pos);
                for (dy, dx) in KNIGHT_DELTAS {
                    let res_coord = (coord.0 + dy, coord.1 + dx);
                    let new_pos = coord_to_pos(res_coord);
                    if res_coord.0 >= 0 && res_coord.0 < 8 && res_coord.1 >= 0 && res_coord.1 < 8 {
                        self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos, piece, captured_piece: state.get_state(player.opp()).get_piece_at_pos(new_pos) }, depth);
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
                    depth,
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
                    depth,
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
                    depth,
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
                let coord = pos_to_coord(prev_pos);
                    
                for (dy, dx) in KING_DELTAS {
                    let res_coord = (coord.0 + dy, coord.1 + dx);
                    let new_pos = coord_to_pos(res_coord);

                    // square is valid 
                    // check for square under attack by opposing piece is done during tree search
                    if res_coord.0 >= 0 && res_coord.0 < 8 && res_coord.1 >= 0 && res_coord.1 < 8 {
                        self.emit_move::<TYPE>(Move::Move { prev_pos, new_pos, piece, captured_piece: state.get_state(player.opp()).get_piece_at_pos(new_pos) }, depth);
                    }
                }
                
                let player_state = state.get_state(player);
                let offset = if player == Player::White {0} else {7};

                // optimization
                if (player_state.meta.can_castle_short && self.opp_move_grid | (1 << (4 + offset)) | (1 << (5 + offset)) | (1 << (6 + offset)) == 0) || 
                (player_state.meta.can_castle_long && self.opp_move_grid | (1 << (4 + offset)) | (1 << (3 + offset)) | (1 << (2 + offset)) == 0){
                    self.opp_move_grid = 0;
                    self.get_all_moves::<false>(state, state.player.opp(), depth);

                    if player_state.meta.can_castle_short && self.opp_move_grid | (1 << (4 + offset)) | (1 << (5 + offset)) | (1 << (6 + offset)) == 0 {
                        self.emit_move::<TYPE>(Move::Castle { is_short:true }, depth)
                    }
                    if player_state.meta.can_castle_long && self.opp_move_grid | (1 << (4 + offset)) | (1 << (3 + offset)) | (1 << (2 + offset)) == 0 {
                        self.emit_move::<TYPE>(Move::Castle { is_short:false }, depth)
                    }
                }
            }
        }
    }
    // overrides move_buf
    // todo: en-passant
    fn get_moves_by_piece_type<const TYPE: bool>(
        &mut self,
        state: &GameState,
        piece: Piece,
        player: Player,
        depth: usize,
    ) {
        for i in 0u8..64 {
            if state.get_state(player).pieces[piece as usize].0 & (1u64 << i) > 0 {
                self.get_move_by_piece::<TYPE>(state, piece, i, player, depth);
            }
        }
    }

    fn get_all_moves<const TYPE: bool>(&mut self, state: &GameState, player: Player, depth: usize) {
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Pawn, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Knight, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Bishop, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Rook, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::Queen, player, depth);
        self.get_moves_by_piece_type::<TYPE>(state, Piece::King, player, depth);
    }

    // fn recurse(
    //     &mut self,
    //     state: &mut GameState,
    //     value: &mut f32,
    //     alpha: &mut f32,
    //     beta: &mut f32,
    //     rem_depth: usize,
    //     last_capture: Option<u8>
    // ) {
        
    // }
    // returns (score, initial piece pos, move piece pos)
    fn calc(&mut self, state: &mut GameState, mut alpha: f32, mut beta: f32, depth: usize, last_move_pos: u8) -> f32 {
        self.nodes_explored += 1;
        if self.nodes_explored % 100000 == 0{
            println!("explored {} nodes, alpha: {}, beta: {}", self.nodes_explored, alpha, beta);
        }

        let score = state.score();
        if depth == MAX_DEPTH_HARD || !(-500_000_000..=500_000_000).contains(&score) {
            return score as f32;
        }

        let mut value = if state.player == Player::White {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        };

        // king under attack
        /*if state.get_state(state.player).pieces[Piece::King as usize].0
            & self.opp_move_grid
            > 0
        {
            self.move_buf.clear();
            self.get_moves_by_piece_type::<false>(&state, Piece::King, state.player, rem_depth);
            // todo: recurse
            self.recurse(&state, value, &mut alpha, &mut beta, rem_depth)
        } else {*/
        self.move_buf[depth].clear();
        self.get_all_moves::<true>(state, state.player, depth);
        self.move_buf[depth].sort_by_key(|a| a.get_cmp_key(last_move_pos));

        while let Some(val) = self.move_buf[depth].pop() {
            // do the state transition
            
            if depth > MAX_DEPTH_SOFT{
                // after soft cap, non-captures are not permitted
                match val{
                    Move::Move { captured_piece: Some(_), new_pos, .. } => {
                        if depth > MAX_DEPTH_CAPTURE && new_pos != last_move_pos{
                            return score as f32
                        }
                    },
                    _ => return score as f32,
                }
            }

            let meta = (state.states[0].meta, state.states[1].meta);
            state.advance_state(val);

            // update meta values
            let next_val = self.calc(state, alpha, beta, depth + 1, if let Move::Move { new_pos, ..} = val {new_pos} else {64});

            state.revert_state(val);

            // revert metadata: easiest to do this way
            state.states[0].meta = meta.0;
            state.states[1].meta = meta.1;

            if state.player == Player::White {
                value = f32::max(value, next_val);
                if value >= beta {
                    break;
                }
                alpha = f32::max(alpha, value);
            } else {
                value = f32::min(value, next_val);
                if value <= alpha {
                    break;
                }
                beta = f32::min(beta, value);
            }
        }
        /*}*/

        value
    }

    pub fn solve(&mut self, state: &mut GameState) -> f32 {
        self.calc(
            state,
            f32::NEG_INFINITY,
            f32::INFINITY,
            0,
            64
        )
    }
}
