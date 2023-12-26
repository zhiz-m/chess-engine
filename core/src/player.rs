use serde::{de::Error, Deserialize};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Player {
    White = 0,
    Black = 1,
}

impl Player {
    pub fn opp(self) -> Player {
        if self == Player::White {
            Player::Black
        } else {
            Player::White
        }
    }
}

impl TryFrom<char> for Player {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value.to_ascii_lowercase() {
            'w' => Player::White,
            'b' => Player::Black,
            _ => return Err("invalid player char"),
        })
    }
}

impl TryFrom<&str> for Player {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "white" => Player::White,
            "black" => Player::Black,
            _ => return Err("invalid player string"),
        })
    }
}

impl<'de> Deserialize<'de> for Player {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "white" => Ok(Player::White),
            "black" => Ok(Player::Black),
            _ => Err(D::Error::custom("invalid player")),
        }
    }
}
