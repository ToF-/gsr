use walkdir::WalkDir;
use std::io::{Result, Error, ErrorKind};

const VALID_EXTENSIONS: [&'static str; 6] = ["jpg", "jpeg", "png", "JPG", "JPEG", "PNG"];
const SELECTION_FILE_NAME: &str = "selections";

pub const THUMB_SUFFIX: &str = "THUMB";
pub const IMAGE_DATA: &str = "IMAGE_DATA";


pub fn get_picture_file_paths(directory: &str) -> Result<Vec<String>> {
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
}


#[test]
fn get_all_pictures_including_sub_folders_except_thumbnails() {
    let result = get_picture_file_paths("testdata");
    assert_eq!(true, result.is_ok());
    let file_paths = result.unwrap();
    assert_eq!(10, file_paths.len());
}
