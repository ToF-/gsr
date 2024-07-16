use image::DynamicImage;
use palette_extract::{get_palette_rgb};

pub type Palette = [u32;9];

pub fn get_palette(image: DynamicImage) -> Palette {
    let mut palette: Palette = [0;9];
    let pixels: &[u8] = image.as_bytes();
    let colors = get_palette_rgb(&pixels);
    colors.iter().enumerate().for_each(|(i,c)| {
        palette[i] = (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32;
    });
    palette.sort();
    palette
}

