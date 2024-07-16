use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum Rank {
   ThreeStars, TwoStars, OneStar, NoStar,
}
