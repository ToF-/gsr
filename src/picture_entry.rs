use std::time::SystemTime;
use crate::rank::Rank;

pub struct PictureEntry {
    file_path: String,
    file_size: u64,
    colors: usize,
    modified_time: SystemTime,
    rank: Rank,
}

pub fn make_picture_entry(file_path: String, file_size: u64, colors: usize, modified_time: SystemTime, rank: Rank) -> PictureEntry {
    PictureEntry {
        file_path: file_path,
        file_size: file_size,
        colors: colors,
        modified_time: modified_time,
        rank: rank,
}
    }

