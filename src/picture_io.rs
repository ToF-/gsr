use std::io::{Result, Error, ErrorKind};
use std::time::SystemTime;
use std::path::{Path,PathBuf};
use crate::path::image_data_file_path;
use std::fs;
use std::fs::{File, read_to_string};
use crate::rank::Rank;
use crate::image_data::ImageData;
use crate::palette::{Colors, get_colors, Palette, get_palette};

pub type FileSize = u64;

pub fn read_file_info(file_path: &str) -> Result<(FileSize, SystemTime)> {
   let path = PathBuf::from(file_path);
   match fs::metadata(path.clone()) {
       Ok(metadata) => {
           let file_size = metadata.len();
           let modified_time = metadata.modified().unwrap();
           Ok((file_size, modified_time))
       },
       Err(err) => Err(err),
   }
}


pub fn read_or_create_image_data(file_path: &str) -> Result<ImageData> {
    let image_data_file_path = image_data_file_path(file_path);
    match read_image_data(&image_data_file_path) {
        Ok(image_data) => Ok(image_data),
        Err(err) => {
            match read_file_info(file_path) {
                Ok((file_size, system_time)) => {
                    match get_palette_from_picture(file_path) {
                        Ok((palette, colors)) => {
                            let image_data = ImageData{
                                colors: colors,
                                rank: Rank::NoStar,
                                selected: false,
                                palette: palette,
                                label: String::from(""),
                            };
                            match write_image_data(&image_data, &image_data_file_path) {
                                Ok(()) => Ok(image_data),
                                Err(err) => Err(err),
                            }
                        },
                        Err(err) => Err(err),
                    }
                },
                Err(err) => Err(err),
            }
        }
    }
}
pub fn read_image_data(file_path: &str) -> Result<ImageData> {
    let path = Path::new(&file_path);
    if path.exists() {
        match read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(image_data) => Ok(image_data),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    } else {
        Err(Error::new(ErrorKind::Other, format!("image_data {} not found", file_path)))
    }
}

pub fn write_image_data(image_data: &ImageData, file_path: &str) -> Result<()> {
    let path = Path::new(&file_path);
    match File::create(path) {
        Ok(file) => {
            match serde_json::to_writer(file, &image_data) {
                Ok(_) => Ok(()),
                Err(err) => Err(err.into()),
            }
        },
        Err(err) => Err(err),
    }
}

pub fn get_palette_from_picture(file_path: &str) -> Result<(Palette,Colors)> {
    let image = image::open(file_path).expect("can't open image file for palette extraction");
    let palette = get_palette(&image);
    let colors = get_colors(&image);
    Ok((palette,colors))
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime,Utc};

    #[test]
    fn get_palette_from_a_picture_file() {
        let result = get_palette_from_picture("testdata/nature/flower.jpg");
        let expected_palette: Palette = [ 0x9c8474, 0xaf382d, 0xccbcb4, 0xd4ab3e, 0xde777a, 0xde978a, 0xe3acb8, 0xeacac0, 0xfbfbfb];
        let expected_colors = 37181; 
        assert_eq!(true, result.is_ok());
        assert_eq!((expected_palette, expected_colors), result.unwrap());
    }
    
    #[test]
    fn read_image_data_deserializes_image_data() {
        let result = read_image_data("testdata/nature/flower-copyIMAGE_DATA.json");
        let expected = ImageData {
            colors: 37181,
            rank: Rank::NoStar,
            selected: false,
            palette: [ 0x9c8474, 0xaf382d, 0xccbcb4, 0xd4ab3e, 0xde777a, 0xde978a, 0xe3acb8, 0xeacac0, 0xfbfbfb],
            label: String::from(""),
        };
        println!("{:?}", result);
        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn set_new_image_data() {
        let expected = ImageData {
            colors: 9,
            rank: Rank::ThreeStars,
            selected: true,
            palette: [0, 1, 2, 3, 4, 5, 6, 7, 8],
            label: String::from("foo"),
        };
        let saved = write_image_data(&expected, "testdata/dummyIMAGE_DATA.json");
        assert_eq!(true, saved.is_ok());
        let result = read_image_data("testdata/dummyIMAGE_DATA.json");
        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn read_picture_file_info_read_file_size_and_modified_time() {
        let result = read_file_info("testdata/nature/flower.jpg");
        assert_eq!(true, result.is_ok());
        let (file_size, modified_time) = result.unwrap();
        assert_eq!(36287, file_size);
        let date_time: DateTime<Utc> = DateTime::from(modified_time);
        let formatted_date = date_time.to_rfc3339();
        assert_eq!(String::from("2024-07-17T19:21:20.047001954+00:00"), formatted_date);
    }
}
