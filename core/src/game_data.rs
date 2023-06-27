use rand::Rng;
use serde::{Deserialize, Deserializer};
use serde::de::Error;

use crate::{
    config::{HashType, NUM_PIECES, PIECE_SCORES},
    types::{Move, Piece, Player},
    util::{self, pos_to_coord, coord_to_pos},
};
use std::collections::BTreeMap;
use std::hash::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid(pub u64);

impl<'a, T> From<T> for Grid
where
    T: IntoIterator<Item = &'a str>,
{
    fn from(value: T) -> Self {
        let mut grid = 0;
        for pos in value {
            grid |= 1u64 << util::canonical_to_pos(pos);
        }
        Grid(grid)
    }
}

impl Grid{
    pub fn contains_pos(&self, pos: u8) -> bool{
        self.0 & (1 << pos) > 0
    }
}

// cheap, copyable player state
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PlayerMetadata {
    pub can_castle_short: bool,
    pub can_castle_long: bool,
}

impl Default for PlayerMetadata{
    fn default() -> Self {
        Self { can_castle_short: true, can_castle_long: true }
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct PlayerState {
    #[serde(deserialize_with = "from_string_vec")]
    pub pieces: [Grid; NUM_PIECES],
    #[serde(default)]
    pub piece_grid: u64,
    #[serde(default)]
    pub meta: PlayerMetadata,
}

fn from_string_vec<'de, D>(deserializer: D) -> Result<[Grid; NUM_PIECES], D::Error>
where
    D: Deserializer<'de>,
{
    let pieces: BTreeMap::<String, Vec<String>> = Deserialize::deserialize(deserializer)?;

    let mut res = [Grid(0); NUM_PIECES];

    for (piece, pos) in pieces.into_iter(){
        let piece: Piece = piece.as_str().try_into().map_err(D::Error::custom)?;
        let grid: Grid = pos.iter().map(|x|x.as_str()).into();
        res[piece as usize] = grid;
    }

    Ok(res)
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
            meta: PlayerMetadata::default(),
        }
    }

    pub fn new_from_state(pos: Vec<Vec<&str>>) -> Option<PlayerState> {
        if pos.len() != 6{
            return None;
        }
        let pieces: [Grid; 6] = pos
            .into_iter()
            .map(Into::<Grid>::into)
            .collect::<Vec<Grid>>()
            .try_into()
            .unwrap();
        let mut res = PlayerState {
            pieces,
            piece_grid: 0,
            meta: PlayerMetadata::default(),
        };
        res.setup();

        Some(res)
    }

    pub fn setup(&mut self){
        self.piece_grid = 0;
        for grid in self.pieces.iter().take(NUM_PIECES) {
            self.piece_grid |= grid.0;
        }
    }

    pub fn square_occupied(&self, pos: u8) -> bool {
        self.piece_grid & (1u64 << pos) > 0
        // self.pieces
        //     .iter()
        //     .map(|x| x.0 & (1u64 << pos) > 0)
        //     .any(|x| x)
    }

    pub fn get_piece_at_pos(&self, pos: u8) -> Option<Piece> {
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

pub struct ZoboristState {
    pieces: [[[HashType; 64]; 6]; 2], // index: player, piece, position
    is_black: HashType,
    castle: [[[[HashType; 2]; 2]; 2]; 2], // index: white castle short, white castle long, black castle short, black castle long
}

impl ZoboristState {
    pub fn new(_seed: u64) -> Self {
        let mut rng = rand::thread_rng();

        let mut state = ZoboristState {
            pieces: [[[0; 64]; 6]; 2],
            is_black: 0,
            castle: [[[[0; 2]; 2]; 2]; 2],
        };

        for num in state.pieces.iter_mut().flatten().flatten() {
            *num = /*((rng.gen::<u64>() as HashType) << 64) + */rng.gen::<u64>() as HashType;
        }

        state.is_black = /*((rng.gen::<u64>() as HashType) << 64) +*/ rng.gen::<u64>() as HashType;

        for num in state.castle.iter_mut().flatten().flatten().flatten() {
            *num = /*((rng.gen::<u64>() as HashType) << 64) + */rng.gen::<u64>() as HashType;
        }

        state
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct GameState {
    #[serde(deserialize_with = "from_player_vec")]
    pub states: [PlayerState; 2],
    pub player: Player,
    #[serde(default)]
    pub hash: HashType,
}

fn from_player_vec<'de, D>(deserializer: D) -> Result<[PlayerState; 2], D::Error>
where
    D: Deserializer<'de>,
{
    let states: BTreeMap::<String, PlayerState> = Deserialize::deserialize(deserializer)?;

    let mut res = [PlayerState::new(Player::White), PlayerState::new(Player::White)];

    for (player, state) in states.into_iter(){
        let player: Player = player.as_str().try_into().map_err(D::Error::custom)?;
        res[player as usize] = state;
    }

    Ok(res)
}

impl Hash for GameState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
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
            hash: 0,
        }
    }

    pub fn check_move_legal(&self, mov: Move) -> bool{
        let state = self.get_state(self.player);
        let opp_state = self.get_state(self.player.opp());
        match mov{
            Move::Move { prev_pos, new_pos, piece, captured_piece: None } => {
               state.pieces[piece as usize].contains_pos(prev_pos) && !opp_state.square_occupied(new_pos) 
            },
            Move::Move { prev_pos, new_pos, piece, captured_piece: Some(captured_piece) } => {
                state.pieces[piece as usize].contains_pos(prev_pos) && opp_state.pieces[captured_piece as usize].contains_pos(new_pos)
            },
            Move::Castle { is_short: true } => state.meta.can_castle_short,
            Move::Castle { is_short: false} => state.meta.can_castle_long,
            Move::PawnPromote { prev_pos, new_pos, captured_piece: None, .. } => {
                state.pieces[Piece::Pawn as usize].contains_pos(prev_pos) && !state.square_occupied(new_pos) && !opp_state.square_occupied(new_pos)
            },
            Move::PawnPromote { prev_pos, new_pos, captured_piece: Some(captured_piece), .. } => {
                state.pieces[Piece::Pawn as usize].contains_pos(prev_pos) && opp_state.pieces[captured_piece as usize].contains_pos(new_pos) 
            },
        }
    }

    pub fn new_from_state(
        white_player_state: PlayerState,
        black_player_state: PlayerState,
        player_to_move: Player,
    ) -> GameState {
        GameState {
            states: [white_player_state, black_player_state],
            player: player_to_move,
            hash: 0,
        }
    }

    pub fn apply_meta_hash(&mut self, zoborist_state: &ZoboristState) {
        self.hash ^= zoborist_state.castle[self.states[0].meta.can_castle_long as usize]
            [self.states[0].meta.can_castle_short as usize]
            [self.states[1].meta.can_castle_long as usize]
            [self.states[1].meta.can_castle_short as usize];
    }

    pub fn apply_piece_hash(
        &mut self,
        zoborist_state: &ZoboristState,
        player: Player,
        piece: Piece,
        pos: u8,
    ) {
        self.hash ^= zoborist_state.pieces[player as usize][piece as usize][pos as usize];
    }

    pub fn change_player(&mut self, zoborist_state: &ZoboristState) {
        self.hash ^= zoborist_state.is_black;
        self.player = self.player.opp();
    }

    fn apply_piece_move(
        &mut self,
        zoborist_state: &ZoboristState,
        player: Player,
        piece: Piece,
        pos: u8,
    ) {
        self.apply_piece_hash(zoborist_state, player, piece, pos);
        let player_state: &mut PlayerState = &mut self.states[player as usize];

        player_state.pieces[piece as usize].0 ^= 1u64 << pos;
        player_state.piece_grid ^= 1u64 << pos;
        // self.hash ^= zoborist_state.pieces[player as usize][piece as usize][pos as usize];
    }

    // zoborist hash
    fn slow_compute_hash(&mut self, zoborist_state: &ZoboristState) {
        self.apply_meta_hash(zoborist_state);
        for player in [Player::White, Player::Black] {
            for piece in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
                Piece::King,
            ] {
                for pos in 0..64 {
                    if self.get_state(player).pieces[piece as usize].0 & (1 << pos) > 0 {
                        self.apply_piece_hash(zoborist_state, player, piece, pos);
                    }
                }
            }
        }
    }

    pub fn setup(&mut self, zoborist_state: &ZoboristState){
        self.states.iter_mut().for_each(|x|x.setup());
        self.slow_compute_hash(zoborist_state);
    }

    pub fn get_state(&self, player: Player) -> &PlayerState {
        &self.states[player as usize]
    }

    // fn get_state_mut(&mut self, player: Player) -> &mut PlayerState {
    //     &mut self.states[player as usize]
    // }

    fn modify_state<const APPLY_METADATA_CHANGES: bool>(
        &mut self,
        next_move: Move,
        zoborist_state: &ZoboristState,
    ) {
        let offset = if self.player == Player::White { 0 } else { 56 };
        match next_move {
            Move::Move {
                prev_pos,
                new_pos,
                piece,
                captured_piece,
            } => {
                // capture/uncapture the piece if any
                if let Some(captured_piece) = captured_piece {
                    self.apply_piece_move(
                        zoborist_state,
                        self.player.opp(),
                        captured_piece,
                        new_pos,
                    );
                    // self.get_state_mut(self.player.opp()).pieces[captured_piece as usize].0 ^=
                    //     1 << new_pos;
                    // self.get_state_mut(self.player.opp()).piece_grid ^= 1 << new_pos;
                }

                // move the current piece
                // self.get_state_mut(self.player).pieces[piece as usize].0 ^=
                //     (1 << prev_pos) | (1 << new_pos);
                // self.get_state_mut(self.player).piece_grid ^= (1 << prev_pos) | (1 << new_pos);
                self.apply_piece_move(zoborist_state, self.player, piece, prev_pos);
                self.apply_piece_move(zoborist_state, self.player, piece, new_pos);

                if APPLY_METADATA_CHANGES {
                    let player_state = &mut self.states[self.player as usize];
                    if piece == Piece::King || (piece == Piece::Rook && prev_pos == offset) {
                        player_state.meta.can_castle_long = false;
                    }
                    if piece == Piece::King || (piece == Piece::Rook && prev_pos == 7 + offset) {
                        player_state.meta.can_castle_short = false;
                    }
                }
            }
            Move::Castle { is_short } => {
                let player_state = &mut self.states[self.player as usize];
                const KING_POS: u8 = 4;
                if APPLY_METADATA_CHANGES {
                    player_state.meta.can_castle_long = false;
                    player_state.meta.can_castle_short = false;
                }

                let rook_pos = if is_short { 7 } else { 0 };

                // xor with initial rook/king positions
                self.apply_piece_move(zoborist_state, self.player, Piece::King, KING_POS + offset);
                self.apply_piece_move(zoborist_state, self.player, Piece::Rook, rook_pos + offset);
                // player_state.piece_grid ^= (1 << (KING_POS + offset)) | (1 << (rook_pos + offset));
                // player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset);
                // player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset);

                // xor with post-castling rook/king positions
                if is_short {
                    // player_state.piece_grid ^=
                    //     (1 << (KING_POS + offset + 2)) | (1 << (rook_pos + offset - 2));
                    // player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset + 2);
                    // player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset - 2);
                    self.apply_piece_move(
                        zoborist_state,
                        self.player,
                        Piece::King,
                        KING_POS + offset + 2,
                    );
                    self.apply_piece_move(
                        zoborist_state,
                        self.player,
                        Piece::Rook,
                        rook_pos + offset - 2,
                    );
                } else {
                    // player_state.piece_grid ^=
                    //     (1 << (KING_POS + offset - 2)) | (1 << (rook_pos + offset + 3));
                    // player_state.pieces[Piece::King as usize].0 ^= 1 << (KING_POS + offset - 2);
                    // player_state.pieces[Piece::Rook as usize].0 ^= 1 << (rook_pos + offset + 3);
                    self.apply_piece_move(
                        zoborist_state,
                        self.player,
                        Piece::King,
                        KING_POS + offset - 2,
                    );
                    self.apply_piece_move(
                        zoborist_state,
                        self.player,
                        Piece::Rook,
                        rook_pos + offset + 3,
                    );
                }
            }
            Move::PawnPromote {
                prev_pos,
                new_pos,
                promoted_to_piece,
                captured_piece,
            } => {
                // capture/uncapture piece if any
                if let Some(captured_piece) = captured_piece {
                    // self.get_state_mut(self.player.opp()).pieces[captured_piece as usize].0 ^=
                    //     1 << new_pos;
                    // self.get_state_mut(self.player.opp()).piece_grid ^= 1 << new_pos;
                    self.apply_piece_move(
                        zoborist_state,
                        self.player.opp(),
                        captured_piece,
                        new_pos,
                    );
                }

                // let player_state: &mut PlayerState = &mut self.states[self.player as usize];

                // pawn promotion
                // player_state.piece_grid ^= (1 << new_pos) | (1 << prev_pos);
                // player_state.pieces[Piece::Pawn as usize].0 ^= 1 << prev_pos;
                // player_state.pieces[promoted_to_piece as usize].0 ^= 1 << new_pos;
                self.apply_piece_move(zoborist_state, self.player, Piece::Pawn, prev_pos);
                self.apply_piece_move(zoborist_state, self.player, promoted_to_piece, new_pos);
            }
        }
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
    
    pub fn is_in_check(&self) -> bool{
        let player_state = self.get_state(self.player);
        let opp_state = self.get_state(self.player.opp());
        let king_pos = (player_state.pieces[Piece::King as usize].0 - 1).count_ones() as u8;
        let coord_pos = pos_to_coord(king_pos);

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

        for (dy,dx) in KNIGHT_DELTAS.into_iter(){
            let new_coord = (coord_pos.0 + dy, coord_pos.1 + dx);
            if new_coord.0 < 0 || new_coord.0 > 7 || new_coord.1 < 0 || new_coord.1 > 7{
                continue;
            }
            if opp_state.pieces[Piece::Knight as usize].contains_pos(coord_to_pos(new_coord)) {
                return true;
            }
        }

        const BISHOP_DELTAS: [(i8, i8); 4] = [(1, -1), (1, 1), (-1, -1), (-1, 1)];
        const ROOK_DELTAS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, -1), (0, 1)];

        let helper = |deltas, pieces: [Piece; 2]|{
            for (dy, dx) in deltas {
                let mut res_coord = util::pos_to_coord(king_pos);
                res_coord.0 += dy;
                res_coord.1 += dx;
                while res_coord.0 >= 0 && res_coord.0 < 8 && res_coord.1 >= 0 && res_coord.1 < 8 {
                    let new_pos = util::coord_to_pos(res_coord);
                    
                    if pieces.iter().map(|x| opp_state.pieces[*x as usize].contains_pos(new_pos)).any(|x|x){
                        return true;
                    }

                    if player_state.square_occupied(new_pos) || opp_state.square_occupied(new_pos) {
                        break;
                    }
                }
            }
            false
        };

        helper(BISHOP_DELTAS, [Piece::Bishop, Piece::Queen]) || helper(ROOK_DELTAS, [Piece::Rook, Piece::Queen])
    }
}