use anyhow::{anyhow, Result};
use dirs::home_dir;
use std::env;
use std::fs;
use std::io;
use std::path::{Path,PathBuf};
use walkdir::WalkDir;

const VALID_EXTENSIONS: [&'static str; 6] = ["jpg", "jpeg", "png", "JPG", "JPEG", "PNG"];

pub const THUMB_SUFFIX: &str = "THUMB";
pub const IMAGE_DATA: &str = "IMAGE_DATA";
const DEFAULT_DIR :&str    = "images/";
pub const DIR_ENV_VAR: &str = "GALLSHDIR";
pub const DEFAULT_TMP_DIR :&str    = ".";
pub const TMP_ENV_VAR: &str = "GALLSHTMP";
pub const DEFAULT_EXTRACT_LIST_FILE_NAME: &str = "gsr_extract.txt";

pub fn default_extract_list_file() -> Result<String> {
    match home_dir() {
        Some(mut path_buf) => {
            path_buf.push(DEFAULT_EXTRACT_LIST_FILE_NAME);
            Ok(path_buf.display().to_string())
        },
        None => Err(anyhow!("cannot open home directory")),
    }
}

pub fn is_valid_directory(dir: &str) -> bool {
    let path = PathBuf::from(dir);
    if ! path.exists() {
       return false
    } else {
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.is_dir()
        } else {
            return false
        }
    }
}
pub fn is_thumbnail(file_name: &str) -> bool {
   file_name.contains(&THUMB_SUFFIX)
}

pub fn check_path(source: &str) -> Result<PathBuf> {
    let path = PathBuf::from(source);
    if !path.exists() {
        Err(anyhow!(format!("directory {} doesn't exist", source)))
    } else {
        match fs::metadata(path.clone()) {
            Ok(metadata) => if metadata.is_dir() {
                Ok(path)
            } else {
                Err(anyhow!(format!("{} is not a directory", source)))
            },
            Err(err) => Err(anyhow!(err)),
        }
    }
}

pub fn interactive_check_path(dir: &str) -> Result<PathBuf> {
    let path = PathBuf::from(dir);
    if !path.exists() {
        println!("directory {} doesn't exist. Create ?", dir);
        let mut response = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut response).expect("can't read from stdin");
        match response.chars().next() {
            Some(ch) if ch == 'y' || ch == 'Y' => {
                match fs::create_dir(path.clone()) {
                    Ok(()) => Ok(path),
                    Err(err) => return Err(anyhow!(err)),
                }
            },
            _ => Err(anyhow!("directory creation cancelled")),
        }
    } else {
        if is_valid_directory(dir) {
            Ok(path)
        } else {
            Err(anyhow!(format!("path {} doesn't exist", dir)))
        }
    }
}

pub fn interactive_check_label_path(target_parent: &str, label: &str) -> Result<PathBuf> {
    let path = PathBuf::from(target_parent).join(label);
    interactive_check_path(path.to_str().unwrap())
}

pub fn check_file(source: &str) -> Result<PathBuf> {
    let path = PathBuf::from(source);
    if !path.exists() {
        Err(anyhow!(format!("file {} doesn't exist", source)))
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
                    Err(anyhow!(format!("{} is not a valid file", source)))
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }
}

pub fn check_reading_list_file(source: &str) -> Result<PathBuf> {
    let path = PathBuf::from(source);
    match path.try_exists() {
        Ok(true) => if path.is_file() {
            Ok(path)
        } else {
            Err(anyhow!(format!("{} is not a valid file", source)))
        },
        Ok(false) => Err(anyhow!(format!("file {} doesn't exist", source))),
        Err(err) => Err(anyhow!(format!("file {} error : {}", source, err))),

    }
}

pub fn get_picture_file_paths(source: &str) -> Result<Vec<String>> {
    let mut picture_number: usize = 0;
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
                        picture_number += 1;
                        println!("{}", picture_number);
                        file_paths.push((&path.display()).to_string())
                    }
                };
            Ok(file_paths.clone())
        },
        Err(err) => Err(err),
    }
}

pub fn file_path_directory(source: &str) -> String {
    let path = Path::new(source);
    path.parent().expect("can't get file_path parent").display().to_string()
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

pub fn temp_directory() -> String {
    let tmp_dir = env::var(TMP_ENV_VAR);
    if let Ok(dir) = &tmp_dir {
        String::from(dir)
    } else {
        String::from(DEFAULT_TMP_DIR)
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
