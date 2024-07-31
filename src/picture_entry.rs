use std::io::{Result};
use std::path::PathBuf;
use std::cmp::Ordering;
use std::cmp::Ordering::*;
use std::time::SystemTime;
use crate::rank::Rank;
use crate::image_data::ImageData;
use crate::picture_io::{copy_file_to_target_directory, delete_file, write_image_data, read_or_create_image_data, read_file_info};
use crate::path::{THUMB_SUFFIX, image_data_file_path, temp_directory};

#[derive(Clone, Debug)]
pub struct PictureEntry {
    pub file_path: String,
    pub file_size: u64,
    pub colors: usize,
    pub modified_time: SystemTime,
    pub rank: Rank,
    pub palette: [u32;9],
    pub label: String,
    pub selected: bool,
    pub deleted: bool,
}

pub fn make_picture_entry(file_path: String, file_size: u64, colors: usize, modified_time: SystemTime, rank: Rank, palette_option: Option<[u32;9]>, label_option: Option<String>, selected: bool) -> PictureEntry {
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
        selected: selected,
        deleted: false,
    }
}

impl PictureEntry {

    pub fn from_file(file_path: &str) -> Result<Self> {
        match read_file_info(file_path) {
            Ok((file_size, modified_time)) => {
                match read_or_create_image_data(file_path) {
                    Ok(image_data) => Ok(make_picture_entry(file_path.to_string(),
                            file_size,
                            image_data.colors,
                            modified_time,
                            image_data.rank,
                            Some(image_data.palette),
                            Some(image_data.label),
                            image_data.selected)),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn original_file_name(&self) -> String {
        let original = &self.file_path;
        let path = PathBuf::from(original);
        path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn original_file_path(&self) -> String {
        if !self.file_path.contains(&THUMB_SUFFIX) {
            self.file_path.clone()
        } else {
            let path = PathBuf::from(self.file_path.clone());
            let parent = path.parent().unwrap();
            let extension = path.extension().unwrap();
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let new_file_stem = match file_stem.strip_suffix("THUMB") {
                Some(s) => s,
                None => &file_stem,
            };
            let new_file_name = format!("{}.{}", new_file_stem, extension.to_str().unwrap());
            let new_path = parent.join(new_file_name);
            new_path.to_str().unwrap().to_string()
        }
    }

    pub fn thumbnail_file_path(&self) -> String {
        if self.file_path.contains(&THUMB_SUFFIX) {
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

    pub fn unlabel(&mut self) {
        self.label = String::from("");
    }

    pub fn set_rank(&mut self, rank: Rank) {
        self.rank = rank
    }

    pub fn cmp_rank(&self, other: &PictureEntry) -> Ordering {
        let cmp = (self.rank.clone() as usize).cmp(&(other.rank.clone() as usize));
        if cmp == Equal {
            self.original_file_path().cmp(&other.original_file_path())
        } else {
            cmp
        }
    }

    pub fn delete_files(&self) {
        delete_file(&self.original_file_path());
        delete_file(&self.thumbnail_file_path());
        delete_file(&self.image_data_file_path());
    }

    pub fn copy_files(&self, target_dir: &str) {
        copy_file_to_target_directory(&self.original_file_path(), target_dir);
        copy_file_to_target_directory(&self.thumbnail_file_path(), target_dir);
        copy_file_to_target_directory(&self.image_data_file_path(), target_dir);
    }

    pub fn copy_file_to_current_dir(&self) -> Result<u64> {
        copy_file_to_target_directory(&self.original_file_path(), &temp_directory())
    }

    pub fn save_image_data(&self) -> Result<()> {
        let image_data = ImageData {
            colors: self.colors,
            rank: self.rank.clone(),
            selected: self.selected,
            palette: self.palette,
            label: self.label.clone(),
        };
        let image_data_file_path = self.image_data_file_path();
        write_image_data(&image_data, &image_data_file_path)
    }

    pub fn label_display(&self, has_focus: bool) -> String {
        format!("{}{}{}{}{}",
            if has_focus { "▄" } else { "" },
            self.rank.show(),
            if self.selected { "△" } else { "" },
            if self.label.len() > 0 {
                format!("{}", self.label)
            } else { String::from("") } ,
            if self.deleted { "🗑" } else { "" },
        )

    }
    pub fn title_display(self) -> String {
        format!("{} {} [{} {} {}] {} {}",
            self.original_file_name(),
            if self.selected { "△" } else { "" },
            self.file_size,
            self.colors,
            self.rank.show(),
            if let Some(s) = self.label() {
                s
            } else {
                String::from("")
            },
            if self.deleted { "🗑" } else { ""})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use std::fs::{copy, remove_file};

    fn my_entry(file_path: &str) -> PictureEntry {
        let day: SystemTime = DateTime::parse_from_rfc2822("Sun, 1 Jan 2023 10:52:37 GMT").unwrap().into();
        make_picture_entry(String::from(file_path), 100, 5, day, Rank::NoStar, None, None, false)
    }

    #[test]
    fn original_file_name_is_the_file_path_without_folders() {
        let entry = my_entry("photos/foo.jpeg");
        assert_eq!(String::from("foo.jpeg"), entry.original_file_name());
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
        entry.set_label(String::from("foo"));
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
    #[test]
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

