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

impl From<i64> for Rank {
    fn from(n: i64) -> Self {
        match n {
            0 => Rank::ThreeStars,
            1 => Rank::TwoStars,
            2 => Rank::OneStar,
            _ => Rank::NoStar,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_and_to_i64() {
        let result: i64 = Rank::OneStar.into();
        assert_eq!(2, result); 
        let rank: Rank = Rank::from(1);
        assert_eq!(Rank::TwoStars, rank);
    }
}
