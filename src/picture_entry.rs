use std::path::PathBuf;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::time::SystemTime;
use crate::rank::Rank;

#[derive(Clone, Debug)]
pub struct PictureEntry {
    pub file_path: String,
    pub file_size: u64,
    pub colors: usize,
    pub modified_time: SystemTime,
    pub rank: Rank,
    pub palette: [u32;9],
    pub label: String
}

pub fn make_picture_entry(file_path: String, file_size: u64, colors: usize, modified_time: SystemTime, rank: Rank, palette_option: Option<[u32;9]>, label_option: Option<String>) -> PictureEntry {
    PictureEntry {
        file_path: file_path,
        file_size: file_size,
        colors: colors,
        modified_time: modified_time,
        rank: rank,
        palette: match palette_option {
            Some(palette) => palette,
            None => [0;9],
        },
        label: match label_option {
            Some(label) => label.clone(),
            None => String::new(),
        },
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

    pub fn label(&self) -> Option<String> {
        if self.label.len() > 0 {
            Some(self.label.clone())
        } else {
            None
        }
    }

    pub fn cmp_label(&self, other: &PictureEntry) -> Ordering {
        match self.label() {
            Some(label_a) => match other.label() {
                Some(label_b) => label_a.cmp(&label_b),
                None => Less,
            },
            None => match other.label() {
                Some(_) => Greater,
                None => Equal,
            },
        }
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn original_file_name_is_the_file_path_without_folders() {
        let day_a: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        let entry = make_picture_entry(String::from("photos/foo.jpeg"), 100, 5, day_a, Rank::NoStar, None, None);
        assert_eq!(entry.original_file_name(), String::from("foo.jpeg"));
    }

}

