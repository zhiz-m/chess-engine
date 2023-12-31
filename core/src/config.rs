// consts: do not change
pub const NUM_PIECES: usize = 6;
pub const MAX_CHILDREN_PER_NODE: usize = 256;

// may be changed
pub type HashType = u64;
pub const KILLER_MOVES_PER_DEPTH: usize = 3;
pub const PIECE_SCORES: [i32; 6] = [1, 3, 3, 5, 9, 2_000_000];
// pub const MOVE_TABLE_BITS: usize = 18;
pub const MOVE_TABLE_SIZE: usize = 1 << 24;
pub const NULL_MOVE_DEPTH_REDUCTION: usize = 2;
pub const NULL_MOVES_PER_BRANCH: u8 = 3;
