use std::fs::copy;
use anyhow::{anyhow, Result};
use dirs::home_dir;
use std::env;
use std::fs;
use std::path::{Path,PathBuf};
use walkdir::WalkDir;

const VALID_EXTENSIONS: [&str; 6] = ["jpg", "jpeg", "png", "JPG", "JPEG", "PNG"];

pub const THUMB_SUFFIX: &str = "THUMB";
pub const IMAGE_DATA: &str = "IMAGE_DATA";
const DEFAULT_DIR :&str    = "images/";
pub const DIR_ENV_VAR: &str = "GALLSHDIR";
pub const DEFAULT_TMP_DIR :&str    = ".";
pub const TMP_ENV_VAR: &str = "GALLSHTMP";
pub const DEFAULT_EXTRACT_LIST_FILE_NAME: &str = "gsr_extract.txt";
pub const ABSOLUTE_PATH: bool = true;

pub fn default_extract_list_file() -> Result<String> {
    match home_dir() {
        Some(mut path_buf) => {
            path_buf.push(DEFAULT_EXTRACT_LIST_FILE_NAME);
            Ok(path_buf.display().to_string())
        },
        None => Err(anyhow!("cannot open home directory")),
    }
}

pub fn check_path(source: &str, absolute: bool) -> Result<PathBuf> {
    let path = PathBuf::from(source);
    if !path.exists() {
        Err(anyhow!(format!("directory {} doesn't exist", source)))
    } else if absolute && !path.has_root() {
        Err(anyhow!(format!("directory {} is relative", source)))
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
                let not_a_thumbnail = matches!(path.to_str().map(|f| f.contains(THUMB_SUFFIX)), Some(false));
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

// recursively collect all file paths from pictures in the <source> folder
// filtering for files with valid extensions (jpeg,jpg,png) and not including "THUMB" in their name 
pub fn get_picture_file_paths(source: &str) -> Result<Vec<String>> {
    match check_path(source, ! ABSOLUTE_PATH) {
        Ok(directory) => {
            let mut file_paths: Vec<String> = Vec::new();
            for path in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()).map(|e| e.into_path()) {
                    let valid_extension = match path.extension() {
                        Some(extension) => VALID_EXTENSIONS.contains(&extension.to_str().unwrap()),
                        None => false,
                    };
                    let not_a_thumbnail = matches!(path.to_str().map(|f| f.contains(THUMB_SUFFIX)), Some(false));
                    if path.is_file() && valid_extension && not_a_thumbnail {
                        file_paths.push((path.display()).to_string())
                    }
                };
            Ok(file_paths.clone())
        },
        Err(err) => Err(err),
    }
}

pub fn copy_all_picture_files(source: &str, target: &str) -> Result<()> {
    if source == target {
        return Err(anyhow!(format!("cannot copy pictures files from {} to {}", source, target)));
    };
    match get_picture_file_paths(source) {
        Ok(file_paths) => {
            for file_path in file_paths {
                let target_file_path = target.to_owned() + "/" + &file_name(&file_path);
                println!("copy {} to {}", file_path, target_file_path);
                match copy(file_path, target_file_path) {
                    Ok(_) => {},
                    Err(err) => return Err(anyhow!(err)),
                }
            };
            Ok(())
        },
        Err(err) => Err(anyhow!(err)),
    }
}

pub fn is_prefix_path(prefix: &str, path: &str) -> bool {
    let mut prefix_components = Path::new(prefix).components();
    let mut path_components = Path::new(path).components();
    loop {
        match (prefix_components.next(), path_components.next()) {
            (Some(p), Some(q)) if p == q => continue,
            (None, _) => return true,
            _ => return false,
        }
    }
}

pub fn file_path_directory(source: &str) -> String {
    let path = Path::new(source);
    path.parent().unwrap_or_else(|| panic!("can't get file_path parent of {}", source))
        .display().to_string()
}

pub fn file_name(source: &str) -> String {
    let path = Path::new(source);
    path.file_name().expect("can't get file_name").to_str().expect("can't convert to str").to_string()
}

pub fn image_data_file_path(original_file_path: &str) -> String {
    let path = PathBuf::from(original_file_path);
    let parent = path.parent().unwrap();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let new_file_name = format!("{}{}.json", file_stem, IMAGE_DATA);
    let new_path = parent.join(new_file_name);
    new_path.to_str().unwrap().to_string()
}

pub fn standard_directory() -> String {
    let gallshdir = env::var(DIR_ENV_VAR);
    if let Ok(standard_dir) = &gallshdir {
        String::from(standard_dir)
    } else {
        String::new()
    }
}

pub fn directory(directory: Option<String>) -> String {
    let gallshdir = env::var(DIR_ENV_VAR);
    if let Some(directory_arg) = directory {
        directory_arg
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

pub fn home_path(file_path: &str) -> String {
    let mut chars = file_path.chars();
    let first_char = chars.next().unwrap();
    if first_char == '~' {
        let remaining = chars.as_str();
        match env::home_dir() {
            Some(home) => home.display().to_string() + remaining,
            None => file_path.to_string()
        }
    } else {
        file_path.to_string()
    }
}

pub fn path_home(file_path: &str) -> String {
    match env::home_dir() {
        None => file_path.to_string(),
        Some(home) => {
            let mut result: String = file_path.to_string();
            let home_path = home.display().to_string();
            result.replace(&home_path, "~")
        },
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn check_is_prefix_path() {
        assert_eq!(true, is_prefix_path("/some/path/prefix", "/some/path/prefix/full"));
    }
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
