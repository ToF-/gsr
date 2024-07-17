use std::io::{Result, Error, ErrorKind};
use std::path::Path;
use std::fs::{File, read_to_string};
use crate::rank::Rank;
use crate::image_data::ImageData;
use crate::palette::{Colors, get_colors, Palette, get_palette};


pub fn get_image_data(file_path: &str) -> Result<ImageData> {
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

pub fn set_image_data(image_data: &ImageData, file_path: &str) -> Result<()> {
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

    #[test]
    fn get_palette_from_a_picture_file() {
        let result = get_palette_from_picture("testdata/nature/flower.jpg");
        let expected_palette: Palette = [ 0x9c8474, 0xaf382d, 0xccbcb4, 0xd4ab3e, 0xde777a, 0xde978a, 0xe3acb8, 0xeacac0, 0xfbfbfb];
        let expected_colors = 37181; 
        assert_eq!(true, result.is_ok());
        assert_eq!((expected_palette, expected_colors), result.unwrap());
    }
    
    #[test]
    fn get_image_data_deserializes_image_data() {
        let result = get_image_data("testdata/nature/flowerIMAGE_DATA.json");
        let expected = ImageData {
            colors: 37181,
            rank: Rank::NoStar,
            selected: false,
            palette: [ 0x9c8474, 0xaf382d, 0xccbcb4, 0xd4ab3e, 0xde777a, 0xde978a, 0xe3acb8, 0xeacac0, 0xfbfbfb],
            label: String::from(""),
        };
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
        let saved = set_image_data(&expected, "testdata/dummyIMAGE_DATA.json");
        assert_eq!(true, saved.is_ok());
        let result = get_image_data("testdata/dummyIMAGE_DATA.json");
        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    pub fn get_picture_file_paths(path: &str) -> Result<Vec<String>> {
        let files: Vec<String> = Vec::new();
        Ok(files.clone())
    }

    #[test]
    fn get_all_pictures_including_sub_folders_except_thumbnails() {
        let result = get_picture_file_paths("testdata");
        assert_eq!(true, result.is_ok());
        let file_paths = result.unwrap();
        assert_eq!(10, file_paths.len());
    }
}
