use std::io::Result;
use palette_extract::{get_palette_rgb};


type Palette = [u32;9];

pub fn get_palette_from_picture(file_path: &str) -> Result<Palette> {
    let mut palette: Palette = [0;9];
    let image = image::open(file_path).expect("can't open image file for palette extraction");
    let pixels = image.as_bytes();
    let colors = get_palette_rgb(&pixels);
    colors.iter().enumerate().for_each(|(i,c)| {
        palette[i] = (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32;
    });
    palette.sort();
    Ok(palette)
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
}
