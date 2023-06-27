

use std::cmp::Ordering;

use crate::{types::{Move, Piece}, config::KILLER_MOVES_PER_DEPTH, Player};

#[derive(Clone,Copy)]
pub struct KillerEntry{
    killer_moves: [Move; KILLER_MOVES_PER_DEPTH],
}

impl Default for KillerEntry{
    fn default() -> Self {
        Self { killer_moves: [Move::Castle { is_short:true }; KILLER_MOVES_PER_DEPTH] }
    }
}

impl KillerEntry{
    pub fn contains(&self, mov: Move) -> bool{
        self.killer_moves.contains(&mov)
    }

    fn insert(&mut self, mov: Move){
        if self.killer_moves[0] == mov{
            return;
        }
        for i in 1..KILLER_MOVES_PER_DEPTH{
            if self.killer_moves[i] == mov{
                self.killer_moves[i] = self.killer_moves[i-1];
                self.killer_moves[i-1] = mov;
                return;
            }
        }
        self.killer_moves[KILLER_MOVES_PER_DEPTH - 1] = mov;
    }
}

pub struct MoveOrderer{
    killer_moves: Vec<KillerEntry>,
    history_table: [[[i32; 64]; 64]; 2]
}

impl MoveOrderer{
    // pub fn contains(&self, mov: Move, depth: usize) -> bool{
    //     self.killer_moves[depth].contains(mov)
    // }
    pub fn new(depth: usize) -> Self{
        const ARRAY: [i32; 64] = [i32::MAX; 64];
        const ARRAY2: [[i32; 64]; 64] = [ARRAY; 64];
        Self { killer_moves: vec![Default::default(); depth+1], history_table: [ARRAY2; 2] }        
    }

    // pub fn get(&self, depth: usize) -> KillerEntry{
    //     self.killer_moves[depth]
    // }

    pub fn insert_killer_move(&mut self, mov: Move, depth: usize){
        self.killer_moves[depth].insert(mov);
    }

    pub fn cmp_move(&mut self, x: Move, y: Move, depth: usize, last_move_pos: u8, player: Player) -> Ordering{
        let cmp1 = self.get_move_cmp_key(x, last_move_pos, self.killer_moves[depth], player);
        cmp1.cmp(&self.get_move_cmp_key(y, last_move_pos, self.killer_moves[depth], player))
    } 

    // note: black castling is the same squares as white castling
    fn get_move_index(mov: Move, player: Player) -> Option<(usize, usize)>{
        match mov{
            Move::Move { prev_pos, new_pos, piece, captured_piece: None } => Some((prev_pos as usize, new_pos as usize)),
            Move::Castle{is_short: true} => if player == Player::White {Some((4, 6))} else {Some((60, 62))},
            Move::Castle{is_short: false} => if player == Player::White {Some((4, 2))} else {Some((60, 58))},
            Move::PawnPromote { prev_pos, new_pos, .. } => Some((prev_pos as usize, new_pos as usize)),
            _ => None
        }        
    }

    // castling ignored
    pub fn update_history(&mut self, mov: Move, player: Player, normalized_depth: usize){
        match Self::get_move_index(mov, player){
            Some((prev_pos, new_pos)) => self.history_table[player as usize][prev_pos][new_pos] -= (normalized_depth as i32) * (normalized_depth as i32),
            None => (),
        }
    }

    fn get_move_cmp_key(&mut self, mov: Move, last_move_pos: u8, killer_entry: KillerEntry, player: Player) -> (u8, i32) {
        match mov{
            Move::Move {
                new_pos,
                piece,
                captured_piece,
                ..
            } => {
                if let Some(captured_piece) = captured_piece {
                    if captured_piece == Piece::King {
                        (0, 0)
                    }
                    // else if piece.value() > captured_piece.value(){
                    //     (5,0)
                    // }
                    else if new_pos == last_move_pos {
                        (1, piece.value() - captured_piece.value())
                    } else if piece.value() < captured_piece.value() {
                        (3, piece.value() - captured_piece.value())
                    }
                    // captures worth the same material are slightly preferred over non-killer silent moves
                    else {(4,piece.value() - captured_piece.value()-1)}
                } 
                else if killer_entry.contains(mov){
                    // println!("killer move");
                    (4,-2)
                }
                else if let Some((prev_pos, new_pos)) = Self::get_move_index(mov, player){
                    (4, self.history_table[player as usize][prev_pos][new_pos])
                }
                else {
                    (4, 0)
                }
            }
            Move::Castle { .. } => (4, 0),
            Move::PawnPromote {
                promoted_to_piece, ..
            } => (2, -promoted_to_piece.value()),
        }
    }

}