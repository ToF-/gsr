use std::io::{Result, Error, ErrorKind};
use std::path::Path;
use std::fs::read_to_string;
use crate::rank::Rank;
use crate::image_data::ImageData;
use crate::palette::{Palette, get_palette};


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

pub fn get_palette_from_picture(file_path: &str) -> Result<Palette> {
    let image = image::open(file_path).expect("can't open image file for palette extraction");
    Ok(get_palette(image))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_palette_from_a_picture_file() {
        let result = get_palette_from_picture("testdata/flower.jpg");
        let expected: Palette = [ 0x9c8474, 0xaf382d, 0xccbcb4, 0xd4ab3e, 0xde777a, 0xde978a, 0xe3acb8, 0xeacac0, 0xfbfbfb];
        assert_eq!(true, result.is_ok());
        assert_eq!(expected, result.unwrap());
    }

    fn get_image_data_from_a_picture_file() {
        let result = get_image_data("testdata/flowerIMAGE_DATA.json");
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
}
