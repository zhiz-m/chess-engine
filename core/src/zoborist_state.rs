use rand::Rng;

use crate::config::HashType;

#[derive(Clone)]
pub struct ZoboristState {
    pub pieces: [[HashType; 64]; 16], 
    pub player: HashType,
    pub castle: [HashType; 16],               
    pub en_passant: [HashType; 9],            // index: row. 8 is no en passant
}

impl ZoboristState {
    pub const STATIC_EMPTY: ZoboristState = ZoboristState {
        pieces: [[0; 64]; 16],
        player: 0,
        castle: [0; 16],
        en_passant: [0; 9],
    };

    pub fn new(_seed: u64) -> Self {
        let mut rng = rand::thread_rng();

        let mut state = Self::STATIC_EMPTY.clone();

        // empty piece, 0, must have no hash
        for num in state.pieces.iter_mut().skip(1).flatten() {
            *num = /*((rng.gen::<u64>() as HashType) << 64) + */rng.gen::<u64>() as HashType;
        }

        state.player = /*((rng.gen::<u64>() as HashType) << 64) +*/ rng.gen::<u64>() as HashType;

        for num in state.castle.iter_mut() {
            *num = /*((rng.gen::<u64>() as HashType) << 64) + */rng.gen::<u64>() as HashType;
        }

        for num in state.en_passant.iter_mut() {
            *num = /*((rng.gen::<u64>() as HashType) << 64) + */rng.gen::<u64>() as HashType;
        }

        state
    }
}