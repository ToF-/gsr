use image::{DynamicImage, GenericImageView, Rgba};
use std::collections::HashSet;
use palette_extract::{get_palette_rgb};

pub type Palette = [u32;9];
pub type Colors = usize;

pub fn get_palette(image: &DynamicImage) -> Palette {
    let mut palette: Palette = [0;9];
    let pixels: &[u8] = image.as_bytes();
    let colors = get_palette_rgb(&pixels);
    colors.iter().enumerate().for_each(|(i,c)| {
        palette[i] = (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32;
    });
    palette.sort();
    palette
}

fn rgba_key(rgba: Rgba<u8>) -> u32 {
    let mut result: u32 = 0;
    for i in 0..4 {
        result <<= 8;
        result |= rgba[i] as u32
    };
    result 
}
pub fn get_colors(image: &DynamicImage) -> usize {
    let iter: Vec<_> = image.pixels().collect();
    let mut colors: HashSet<u32> = HashSet::new();
    for i in iter {
        let rgba = i.2;
        colors.insert(rgba_key(rgba));
    };
    colors.len()
}

