use std::mem::swap;

use crate::{types::Move, config::KILLER_MOVES_PER_DEPTH};

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

pub struct KillerTable<const MAX_DEPTH: usize>{
    killer_moves: [KillerEntry; MAX_DEPTH]
}

impl<const MAX_DEPTH: usize> Default for KillerTable<MAX_DEPTH>{
    fn default() -> Self {
        Self { killer_moves: [Default::default(); MAX_DEPTH] }
    }
}

impl<const MAX_DEPTH: usize> KillerTable<MAX_DEPTH>{
    // pub fn contains(&self, mov: Move, depth: usize) -> bool{
    //     self.killer_moves[depth].contains(mov)
    // }

    pub fn get(&self, depth: usize) -> KillerEntry{
        self.killer_moves[depth]
    }

    pub fn insert(&mut self, mov: Move, depth: usize){
        self.killer_moves[depth].insert(mov);
    }
}