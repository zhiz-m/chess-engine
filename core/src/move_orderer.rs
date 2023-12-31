use std::cmp::Ordering;

use crate::{
    config::KILLER_MOVES_PER_DEPTH, move_buffer_entry::MoveBufferEntry, player::Player,
    types::Move, GameState,
};

#[derive(Clone, Copy)]
pub struct KillerEntry {
    killer_moves: [Move; KILLER_MOVES_PER_DEPTH],
}

impl Default for KillerEntry {
    fn default() -> Self {
        Self {
            killer_moves: [Move::Castle { is_short: true }; KILLER_MOVES_PER_DEPTH],
        }
    }
}

impl KillerEntry {
    pub fn contains(&self, mov: Move) -> bool {
        self.killer_moves.contains(&mov)
    }

    fn insert(&mut self, mov: Move) {
        if self.killer_moves[0] == mov {
            return;
        }
        for i in 1..KILLER_MOVES_PER_DEPTH {
            if self.killer_moves[i] == mov {
                self.killer_moves[i] = self.killer_moves[i - 1];
                self.killer_moves[i - 1] = mov;
                return;
            }
        }
        self.killer_moves[KILLER_MOVES_PER_DEPTH - 1] = mov;
    }
}

pub struct MoveOrderer {
    killer_moves: Vec<KillerEntry>,
    history_table: [[[u32; 64]; 64]; 2],
}

impl MoveOrderer {
    // pub fn contains(&self, mov: Move, depth: usize) -> bool{
    //     self.killer_moves[depth].contains(mov)
    // }
    pub fn new(depth: usize) -> Self {
        const ARRAY: [u32; 64] = [0; 64];
        const ARRAY2: [[u32; 64]; 64] = [ARRAY; 64];
        Self {
            killer_moves: vec![Default::default(); depth + 1],
            history_table: [ARRAY2; 2],
        }
    }

    #[inline(always)]
    pub fn lift_killer_moves(&mut self, depth_to_lift: usize) {
        for _ in 0..depth_to_lift {
            self.killer_moves.insert(0, KillerEntry::default());
        }
    }

    // pub fn get(&self, depth: usize) -> KillerEntry{
    //     self.killer_moves[depth]
    // }
    
    #[inline(always)]
    pub fn insert_killer_move(&mut self, mov: Move, depth: usize) {
        self.killer_moves[depth].insert(mov);
    }

    #[inline(always)]
    pub fn cmp_move(
        &mut self,
        x: &MoveBufferEntry,
        y: &MoveBufferEntry,
        depth: usize,
        last_move_pos: u8,
        player: Player,
        transposition_move: Option<Move>,
    ) -> Ordering {
        let cmp1 = self.get_move_cmp_key(
            x,
            last_move_pos,
            self.killer_moves[depth],
            player,
            transposition_move,
        );
        cmp1.cmp(&self.get_move_cmp_key(
            y,
            last_move_pos,
            self.killer_moves[depth],
            player,
            transposition_move,
        ))
    }

    #[inline(always)]
    // note: black castling is the same squares as white castling
    fn get_move_index(mov: Move, player: Player) -> Option<(u8, u8)> {
        match mov {
            Move::Move {
                prev_pos,
                new_pos,
                pieces,
            } => {
                let (piece, captured_piece) = pieces.to_square_types();
                if captured_piece.is_empty() {
                    None
                } else {
                    Some((prev_pos, new_pos))
                }
            }
            Move::Castle { is_short: true } => {
                if player == Player::White {
                    Some((4, 6))
                } else {
                    Some((60, 62))
                }
            }
            Move::Castle { is_short: false } => {
                if player == Player::White {
                    Some((4, 2))
                } else {
                    Some((60, 58))
                }
            }
            Move::PawnPromote {
                prev_pos, new_pos, ..
            } => Some((prev_pos, new_pos)),
            Move::EnPassant {
                prev_column,
                new_column,
            } => Some(match player {
                Player::White => (32 + prev_column, 40 + new_column),
                Player::Black => (24 + prev_column, 16 + new_column),
            }),
            _ => None,
        }
    }

    // castling ignored
    pub fn update_history(&mut self, mov: Move, player: Player, normalized_depth: usize) {
        match Self::get_move_index(mov, player) {
            Some((prev_pos, new_pos)) => {
                self.history_table[player as usize][prev_pos as usize][new_pos as usize] +=
                    (normalized_depth as u32) * (normalized_depth as u32)
            }
            None => (),
        }
    }

    #[inline(always)]
    pub fn move_is_killer(&self, mov: Move, depth: usize) -> bool {
        self.killer_moves[depth].contains(mov)
    }

    #[inline(always)]
    fn get_move_cmp_key(
        &mut self,
        move_buffer_entry: &MoveBufferEntry,
        last_move_pos: u8,
        killer_entry: KillerEntry,
        player: Player,
        transposition_move: Option<Move>,
    ) -> u32 {
        let mov = move_buffer_entry.get_move();
        match mov {
            Move::Move {
                new_pos, pieces, ..
            } => {
                let (piece, captured_piece) = pieces.to_square_types();
                if captured_piece.is_king() {
                    return 0;
                } else if let Some(trans_mov) = transposition_move {
                    if mov == trans_mov {
                        return 1 << 16;
                    }
                }
                if !captured_piece.is_empty() {
                    // if new_pos == last_move_pos {
                    //     (1, piece.value())
                    // }
                    // else

                    // // if piece.value() <= captured_piece.value()
                    //  {
                    //     (2, piece.value() - captured_piece.value() * 100)
                    // }
                    // captures worth the same material are slightly preferred over non-killer silent moves
                    // else {
                    //     (4, piece.value() - captured_piece.value() * 100 - 1)
                    // }
                    let see = move_buffer_entry.get_see_exn();
                    if see > 0 {
                        let first = if new_pos == last_move_pos { 1 } else { 2 };
                        (first << 16) + (1 << 16) - 1 - captured_piece.value()
                    } else if see == 0 {
                        if new_pos == last_move_pos {
                            3 << 16
                        } else {
                            (3 << 16) + 1
                        }
                    } else {
                        (5 << 16) + (1 << 16) - 1 - see as u32
                    }
                } else if killer_entry.contains(mov) {
                    // println!("killer move");
                    (3 << 16) + 2
                } else if let Some((prev_pos, new_pos)) = Self::get_move_index(mov, player) {
                    (4 << 16) + (1 << 16)
                        - 1
                        - (self.history_table[player as usize][prev_pos as usize][new_pos as usize]
                            & 0xffff)
                } else {
                    (4 << 16) + (1 << 16) - 1
                }
            }
            Move::Castle { .. } => (4 << 16) + (1 << 16) - 10,
            Move::PawnPromote { pieces, .. } => {
                let (promoted_to_piece, _) = pieces.to_square_types();
                (2 << 16) + (1 << 16) - 1 - promoted_to_piece.value()
            }
            Move::EnPassant { .. } => (3 << 16) + 3,
        }
    }
}
