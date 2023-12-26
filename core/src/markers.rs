use crate::player::Player;

pub trait PlayerMarker{
    const IS_WHITE: bool;
    const PLAYER: Player;
}

pub struct WhiteMarker;
impl PlayerMarker for WhiteMarker{
    const IS_WHITE: bool = true;
    const PLAYER: Player = Player::White;
}

pub struct BlackMarker;
impl PlayerMarker for BlackMarker{
    const IS_WHITE: bool = false;
    const PLAYER: Player = Player::Black;
}

#[macro_export]
macro_rules! player_to_marker {
    ($player_expr:expr, $block:block) => {
        {
            use crate::{player::Player, markers::{BlackMarker, WhiteMarker}};
            if $player_expr == Player::White {
                type P = WhiteMarker;
                $block
            }
            else {
                type P = BlackMarker;
                $block
            }
        }
    };
}

pub use player_to_marker;

pub trait CastleTypeMarker {
    const IS_SHORT: bool;
}

pub struct CastleShortMarker;
impl CastleTypeMarker for CastleShortMarker {
    const IS_SHORT: bool = true;
}

pub struct CastleLongMarker;
impl CastleTypeMarker for CastleLongMarker {
    const IS_SHORT: bool = false;
}