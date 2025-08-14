use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Rank {
   ThreeStars, TwoStars, OneStar, NoStar,
}

impl Rank {
    pub fn show(&self) -> String {
        let limit = 3 - *self as usize;
        if limit > 0 {
            "☆".repeat(limit)
        } else {
            "".to_string()
        }
    }

}

impl Into<u64> for Rank {
    fn into(self) -> u64 {
        match self {
            Rank::ThreeStars => 0,
            Rank::TwoStars => 1,
            Rank::OneStar => 2,
            Rank::NoStar => 3,
        }
    }
}
impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Rank::ThreeStars => "☆☆☆",
            Rank::TwoStars => "☆☆",
            Rank::OneStar => "☆",
            Rank::NoStar => "_",
        })
    }
}
