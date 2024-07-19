use walkdir::WalkDir;
use std::fs;
use std::path::PathBuf;

use std::io::{Result, Error, ErrorKind};

const VALID_EXTENSIONS: [&'static str; 6] = ["jpg", "jpeg", "png", "JPG", "JPEG", "PNG"];
const SELECTION_FILE_NAME: &str = "selections";

pub const THUMB_SUFFIX: &str = "THUMB";
pub const IMAGE_DATA: &str = "IMAGE_DATA";


pub fn check_path(source: &str) -> Result<PathBuf> {
    let path = PathBuf::from(source);
    if !path.exists() {
        Err(Error::new(ErrorKind::Other, format!("path {} doesn't exist", source)))
    } else {
        match fs::metadata(path.clone()) {
            Ok(metadata) => if metadata.is_dir() {
                Ok(path)
            } else {
                Err(Error::new(ErrorKind::Other, format!("path {} is not a directory", source)))
            },
            Err(err) => Err(err),
        }
    }
}

pub fn get_picture_file_paths(source: &str) -> Result<Vec<String>> {
    match check_path(source) {
        Ok(directory) => {
            let mut file_paths: Vec<String> = Vec::new();
            for path in WalkDir::new(directory).into_iter().filter_map(|e| e.ok())
                .map(|e| e.into_path()) {
                    let valid_extension = match path.extension() {
                        Some(extension) => VALID_EXTENSIONS.contains(&extension.to_str().unwrap()),
                        None => false,
                    };
                    let not_a_thumbnail = match path.to_str().map(|f| f.contains(THUMB_SUFFIX)) {
                        Some(false) => true,
                        _ => false,
                    };
                    if path.is_file() && valid_extension && not_a_thumbnail {
                        file_paths.push((&path.display()).to_string())
                    }
                };
            Ok(file_paths.clone())
        },
        Err(err) => Err(err),
    }
}

pub fn image_data_file_path(original_file_path: &str) -> String {
    let path = PathBuf::from(original_file_path);
    let parent = path.parent().unwrap();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let new_file_name = format!("{}{}.json", file_stem, IMAGE_DATA);
    let new_path = parent.join(new_file_name);
    new_path.to_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn get_all_pictures_including_sub_folders_except_thumbnails() {
        let result = get_picture_file_paths("testdata");
        assert_eq!(true, result.is_ok());
        let file_paths = result.unwrap();
        assert_eq!(10, file_paths.len());
    }

    #[test]
    fn get_an_error_on_absent_directory() {
        let result = get_picture_file_paths("foo");
        assert_eq!(false, result.is_ok());
        assert_eq!("path foo doesn't exist", result.unwrap_err().to_string());
    }

    #[test]
    fn get_an_error_on_not_a_directory() {
        let result = get_picture_file_paths("testdata/nature/flower.jpg");
        assert_eq!(false, result.is_ok());
        assert_eq!("path testdata/nature/flower.jpg is not a directory", result.unwrap_err().to_string());
    }

}
