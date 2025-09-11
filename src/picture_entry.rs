use crate::image_data::ImageData;
use itertools::Itertools;
use std::collections::HashSet;
use crate::Database;
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::time::SystemTime;
use crate::rank::Rank;
use crate::picture_io::{copy_file_to_target_directory, delete_file, read_or_create_image_data, read_file_info};
use crate::path::{THUMB_SUFFIX, image_data_file_path, temp_directory};

pub type PictureEntries = Vec<PictureEntry>;

#[derive(Clone, Debug)]
pub struct PictureEntry {
    pub file_path: String,
    pub file_size: u64,
    pub modified_time: SystemTime,
    pub deleted: bool,
    pub image_data: ImageData,
}

pub fn make_picture_entry(file_path: String, file_size: u64, modified_time: SystemTime, image_data: ImageData, deleted: bool) -> PictureEntry {
    let data = image_data.clone();
    PictureEntry {
        file_path,
        file_size,
        modified_time,
        deleted,
        image_data: data,
    }
}

impl PictureEntry {

    pub fn from_file_or_database(file_path: &str, database: &Database) -> Result<Self> {
        match database.retrieve_or_insert_picture_entry(file_path) {
            Err(err) => Err(anyhow!(err)),
            Ok(Some(picture_entry)) => Ok(picture_entry),
            Ok(None) => Self::from_file(file_path),
        }
    }

    pub fn from_file(file_path: &str) -> Result<Self> {
        println!("retrieving picture from file {}", file_path);
        match read_file_info(file_path) {
            Ok((file_size, modified_time)) => match read_or_create_image_data(file_path) {
                Ok(image_data) => Ok(make_picture_entry(
                        file_path.to_string(),
                        file_size,
                        modified_time,
                        image_data,
                        false)),
                Err(err) => Err(anyhow!(err)),
            },
            Err(err) => Err(err),
        }
    }

    pub fn parent_path(&self) -> String {
        let original = &self.file_path;
        let path = PathBuf::from(original);
        path.parent().unwrap().to_str().unwrap().to_string()
    }
    pub fn original_file_name(&self) -> String {
        let original = &self.file_path;
        let path = PathBuf::from(original);
        path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn original_file_path(&self) -> String {
        if !self.file_path.contains(THUMB_SUFFIX) {
            self.file_path.clone()
        } else {
            let path = PathBuf::from(self.file_path.clone());
            let parent = path.parent().unwrap();
            let extension = path.extension().unwrap();
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let new_file_stem = match file_stem.strip_suffix("THUMB") {
                Some(s) => s,
                None => file_stem,
            };
            let new_file_name = format!("{}.{}", new_file_stem, extension.to_str().unwrap());
            let new_path = parent.join(new_file_name);
            new_path.to_str().unwrap().to_string()
        }
    }

    pub fn thumbnail_file_path(&self) -> String {
        if self.file_path.contains(THUMB_SUFFIX) {
            self.file_path.to_string()
        } else {
            let path = PathBuf::from(self.file_path.clone());
            let parent = path.parent().unwrap();
            let extension = path.extension().unwrap();
            let file_stem = path.file_stem().unwrap();
            let new_file_name = format!("{}{}.{}", file_stem.to_str().unwrap(), THUMB_SUFFIX, extension.to_str().unwrap());
            let new_path = parent.join(new_file_name);
            new_path.to_str().unwrap().to_string()
        }
    }

    pub fn image_data_file_path(&self) -> String {
        image_data_file_path(&self.original_file_path())
    }


    pub fn equal_content(&self, other:&PictureEntry) -> Result<bool> {
        if self.file_size == 0 {
            return Ok(false);
        }
        if self.file_size != other.file_size {
            return Ok(false);
        };
        match std::fs::read(self.file_path.clone()) {
            Ok(self_bytes) => {
                match std::fs::read(other.file_path.clone()) {
                    Ok(other_bytes) => {
                        for i in 0 .. self_bytes.len() {
                            if self_bytes[i] != other_bytes[i] {
                                return Ok(false)
                            }
                        };
                        Ok(true)
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn label(&self) -> Option<String> {
        if !self.image_data.label.is_empty() {
            Some(self.image_data.label.clone())
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

    pub fn add_tag(&mut self, tag: &str) {
        if !tag.is_empty() {
            self.image_data.tags.insert(tag.to_string());
        }
    }

    pub fn delete_tag(&mut self, tag : &str) {
        self.image_data.tags.remove(tag);
    }

    pub fn set_label(&mut self, label: &str) {
        self.image_data.label = label.to_string()
    }

    pub fn unlabel(&mut self) {
        self.image_data.label = String::from("");
    }

    pub fn set_rank(&mut self, rank: Rank) {
        self.image_data.rank = rank
    }

    pub fn cmp_rank(&self, other: &PictureEntry) -> Ordering {
        let cmp = (self.image_data.rank as usize).cmp(&(other.image_data.rank as usize));
        if cmp == Equal {
            self.original_file_path().cmp(&other.original_file_path())
        } else {
            cmp
        }
    }

    pub fn delete_files(&self) -> Result<()> {
        match delete_file(&self.original_file_path()) {
            Ok(_) => match delete_file(&self.thumbnail_file_path()) {
                Ok(_) => delete_file(&self.image_data_file_path()),
                Err(err) => Err(anyhow!(err)),
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn copy_files(&self, target_dir: &str) -> Result<u64> {
        copy_file_to_target_directory(&self.original_file_path(), target_dir)
            .and_then(|r1| {
                copy_file_to_target_directory(&self.thumbnail_file_path(), target_dir)
                    .and_then(|r2| {
                        copy_file_to_target_directory(&self.image_data_file_path(), target_dir)
                            .map(|r3| r1 + r2 + r3)
                    })
            })
    }

    pub fn copy_picture_file_to_temp(&self) -> Result<u64> {
        copy_file_to_target_directory(&self.original_file_path(), &temp_directory())
    }

    pub fn label_display(&self, has_focus: bool) -> String {
        format!("{}{}{}{}{}",
            if has_focus { "▄" } else { "" },
            self.image_data.rank.show(),
            if self.image_data.selected { "△" } else { "" },
            if !self.image_data.label.is_empty() {
                self.image_data.label.to_string()
            } else { String::from("") } ,
            if self.deleted { "🗑" } else { "" },
        )

    }

    pub fn display_tags(tags: HashSet<String>) -> String {
        match tags.len() {
            0 => String::from(""),
            _ => format!("| {} |", tags.iter().join(" ")),
        }
    }

    pub fn title_display(self) -> String {
        format!("{} {} {} [{} {} {}] {} {} {}",
            if self.image_data.cover { "🌟" } else { "" },
            self.original_file_name(),
            if self.image_data.selected { "△" } else { "" },
            self.file_size,
            self.image_data.colors,
            self.image_data.rank.show(),
            self.label().unwrap_or_default(),
            if self.deleted { "🗑" } else { ""},
            Self::display_tags(self.image_data.tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use std::fs::{copy, remove_file};

    fn my_entry(file_path: &str) -> PictureEntry {
        let day: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        make_picture_entry(String::from(file_path), 100, 5, day, Rank::NoStar, None, None, false, false, false, HashSet::new())
    }

    #[test]
    fn original_file_name_is_the_file_path_without_folders() {
        let entry = my_entry("photos/foo.jpeg");
        assert_eq!(String::from("foo.jpeg"), entry.original_file_name());
    }

    #[test]
    fn parent_path_is_the_file_path_without_file_name() {
        let entry = my_entry("photos/foo/bar/qux.jpeg");
        assert_eq!(String::from("photos/foo/bar"), entry.parent_path());
        let orphan = my_entry("foo.jpeg");
        assert_eq!(String::from(""), orphan.parent_path());

    }
    #[test]
    fn thumbnail_path_is_the_file_path_with_thumbnail_suffix() {
        let entry = my_entry("photos/foo.jpeg");
        assert_eq!(String::from("photos/fooTHUMB.jpeg"), entry.thumbnail_file_path());
    }

    #[test]
    fn original_file_path_is_the_file_path_without_thumb_suffix() {
        let entry = my_entry("photos/fooTHUMB.jpeg");
        assert_eq!(String::from("photos/foo.jpeg"), entry.original_file_path());
    }

    #[test]
    fn image_data_file_path_is_the_file_path_with_json_extension() {
        let entry = my_entry("photos/foo.jpeg");
        assert_eq!(String::from("photos/fooIMAGE_DATA.json"), entry.image_data_file_path());
    }

    #[test]
    fn setting_label() {
        let mut entry = my_entry("photos/foo.jpeg");
        entry.set_label(&String::from("foo"));
        assert_eq!(Some(String::from("foo")), entry.label());
    }

    #[test]
    fn make_picture_entry_from_file_and_image_data_file() {
        let result = PictureEntry::from_file("testdata/nature/flower.jpg");
        assert_eq!(true, result.is_ok());
        let entry = result.unwrap();
        assert_eq!(36287, entry.file_size);
        assert_eq!(37181, entry.colors);
        assert_eq!(10257524, entry.palette[0]);
    }
 //   #[test] file changed
    fn make_picture_entry_from_file_create_image_data_file_if_need_be() {
        let _ = copy("testdata/nature/flowerIMAGE_DATA.json", "testdata/temp");
        let _ = remove_file("testdata/nature/flowerIMAGE_DATA.json");
        let result = PictureEntry::from_file("testdata/nature/flower.jpg");
        let _ = copy("testdata/temp", "testdata/nature/flowerIMAGE_DATA.json");
        assert_eq!(true, result.is_ok());
        let entry = result.unwrap();
        assert_eq!(36287, entry.file_size);
        assert_eq!(37181, entry.colors);
        assert_eq!(10257524, entry.palette[0]);
    }

}

