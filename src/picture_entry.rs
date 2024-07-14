use std::path::PathBuf;
use std::time::SystemTime;
use crate::rank::Rank;

#[derive(Clone)]
pub struct PictureEntry {
    pub file_path: String,
    pub file_size: u64,
    pub colors: usize,
    pub modified_time: SystemTime,
    pub rank: Rank,
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

impl PictureEntry {
    pub fn original_file_name(&self) -> String {
        let original = &self.file_path;
        let path = PathBuf::from(original);
        path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn original_file_path(&self) -> String {
        self.file_path.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn original_file_name_is_the_file_path_without_folders() {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let entry = make_picture_entry(String::from("photos/foo.jpeg"), 100, 5, day_a, Rank::NoStar);
        assert_eq!(entry.original_file_name(), String::from("foo.jpeg"));
    }

}

