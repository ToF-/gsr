use walkdir::WalkDir;
use std::fs;
use std::env;
use std::path::PathBuf;

use std::io::{Result, Error, ErrorKind};

const VALID_EXTENSIONS: [&'static str; 6] = ["jpg", "jpeg", "png", "JPG", "JPEG", "PNG"];

pub const THUMB_SUFFIX: &str = "THUMB";
pub const IMAGE_DATA: &str = "IMAGE_DATA";
const DEFAULT_DIR :&str    = "images/";
pub const DIR_ENV_VAR: &str = "GALLSHDIR";

pub fn is_thumbnail(file_name: &str) -> bool {
   file_name.contains(&THUMB_SUFFIX)
}

pub fn check_path(source: &str) -> Result<PathBuf> {
    let path = PathBuf::from(source);
    if !path.exists() {
        Err(Error::new(ErrorKind::Other, format!("directory {} doesn't exist", source)))
    } else {
        match fs::metadata(path.clone()) {
            Ok(metadata) => if metadata.is_dir() {
                Ok(path)
            } else {
                Err(Error::new(ErrorKind::Other, format!("{} is not a directory", source)))
            },
            Err(err) => Err(err),
        }
    }
}

pub fn check_file(source: &str) -> Result<PathBuf> {
    let path = PathBuf::from(source);
    if !path.exists() {
        Err(Error::new(ErrorKind::Other, format!("file {} doesn't exist", source)))
    } else {
        match fs::metadata(path.clone()) {
            Ok(_) => {
                let valid_extension = match path.extension() {
                    Some(extension) => VALID_EXTENSIONS.contains(&extension.to_str().unwrap()),
                    None => false,
                };
                let not_a_thumbnail = match path.to_str().map(|f| f.contains(THUMB_SUFFIX)) {
                    Some(false) => true,
                    _ => false,
                };
                if path.is_file() && valid_extension && not_a_thumbnail {
                    Ok(path)
                } else {
                    Err(Error::new(ErrorKind::Other, format!("{} is not a valid file", source)))
                }
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

pub fn directory(directory: Option<String>) -> String {
    let gallshdir = env::var(DIR_ENV_VAR);
    if let Some(directory_arg) = directory {
        String::from(directory_arg)
    } else if let Ok(standard_dir) = &gallshdir {
        String::from(standard_dir)
    } else {
        println!("GALLSHDIR variable not set. Using {} as default.", DEFAULT_DIR);
        String::from(DEFAULT_DIR)
    }
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
        assert_eq!("directory foo doesn't exist", result.unwrap_err().to_string());
    }

    #[test]
    fn get_an_error_on_not_a_directory() {
        let result = get_picture_file_paths("testdata/nature/flower.jpg");
        assert_eq!(false, result.is_ok());
        assert_eq!("testdata/nature/flower.jpg is not a directory", result.unwrap_err().to_string());
    }

}
