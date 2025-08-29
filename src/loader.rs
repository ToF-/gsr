use anyhow::{anyhow,Result};
use crate::Database;
use crate::path::check_file;
use crate::picture_entry::{PictureEntries};

pub fn load_picture_entry_from_file_path(database: &Database, file_path: &str) -> Result<PictureEntries> {
    let mut picture_entries:PictureEntries = vec![];
    match check_file(file_path) {
        Ok(_) => match database.retrieve_or_create_image_data(file_path) {
            Ok(Some(picture_entry)) => {
                picture_entries.push(picture_entry);
                Ok(picture_entries)
            },
            Ok(None) => Err(anyhow!("{} image data not found in database'", file_path)),
            Err(err) => Err(anyhow!(err)),
        },
        Err(err) => Err(anyhow!(err)),
    }
}

pub fn load_picture_entries_from_covers(database: &mut Database) -> Result<PictureEntries> {
    database.select_cover_pictures()
}
pub fn load_picture_entries_from_directory_into_db(database: &mut Database, directory: &str, in_std_dir: bool) -> Result<PictureEntries> {
    match database.insert_difference_from_directory(&directory, in_std_dir) {
        Ok(picture_entries) => Ok(picture_entries),
        Err(err) => Err(anyhow!(err)),
    }
}
