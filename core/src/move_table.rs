use crate::{types::Move, config::HashType, GameState};

#[derive(Clone,Copy)]
pub struct MoveEntry{
    pub hash: HashType,
    pub mov: Move, // the move following this position to test for legality
    pub depth: u8,
    pub value: i32
}

impl Default for MoveEntry{
    fn default() -> Self {
        Self { hash: 0, mov: Move::Castle { is_short: false }, depth: u8::MAX, value: 0 }
    }
}

#[derive(Clone, Copy)]
struct MoveTableBucket(MoveEntry, MoveEntry);

impl Default for MoveTableBucket{
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

impl MoveTableBucket{
    fn insert(&mut self, entry: MoveEntry){
        if self.0.depth > entry.depth {
            self.1 = self.0;
            self.0 = entry;
        }
        else {self.1 = entry}
    } 
}

pub struct MoveTable<const MOVE_TABLE_SIZE: usize>{
    table: Vec<MoveTableBucket>,
}

impl<const MOVE_TABLE_SIZE: usize> Default for MoveTable<MOVE_TABLE_SIZE>{
    fn default() -> Self {
        Self { table: vec![MoveTableBucket::default(); MOVE_TABLE_SIZE] }
    }
}

impl<const MOVE_TABLE_SIZE: usize> MoveTable<MOVE_TABLE_SIZE>{
    pub fn get_entry(&self, hash: HashType, depth: u8, state: &GameState) -> Option<MoveEntry>{
        let ind = hash as usize & (MOVE_TABLE_SIZE - 1);
        if self.table[ind].0.depth <= depth && self.table[ind].0.hash == hash && state.check_move_legal(self.table[ind].0.mov){
            Some(self.table[ind].0)
        }
        else if self.table[ind].1.depth <= depth && self.table[ind].1.hash == hash && state.check_move_legal(self.table[ind].1.mov){
            Some(self.table[ind].1)
        } 
        else {None}
    }

    pub fn insert_entry(&mut self, entry: MoveEntry){
        let ind = entry.hash as usize & (MOVE_TABLE_SIZE - 1);
        self.table[ind].insert(entry);
    }
}