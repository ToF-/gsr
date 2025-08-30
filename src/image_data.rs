use crate::palette::Palette;
use serde::{Deserialize, Serialize};
use crate::rank::Rank;


#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub struct ImageData {
    pub colors: usize,
    pub rank: Rank,
    pub selected: bool,
    pub palette: Palette,
    pub label: String,
    pub cover: bool,
}
