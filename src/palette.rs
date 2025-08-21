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

pub fn palette_to_blob(palette: &Palette) -> [u8;36] {
    let mut result: [u8;36] = [0; 36];
    let mut pos: usize = 0;
    for i in 0..9 {
        let mut value: u32 = palette[i];
        for _ in 0..4 {
            result[pos] = (value & 255) as u8;
            pos += 1;
            value = value >> 8;
        }
    }
    return result;
}

pub fn blob_to_palette(blob: &[u8;36]) -> Palette {
    let mut result: Palette = [0;9];
    for i in 0..9 {
        for j in 0..4 {
            let pos:usize = i * 4 + j;
            result[i] = result[i] | (blob[pos] as u32) << j*8;
        }
    }
    result
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_blob_and_vice_versa() {
        let expected: Palette = [0x23174807u32, 0x11223344u32, 0x44332211u32, 0x48072317u32, 0xdeadbeefu32, 0x0a0b0c0du32,0x00000000u32,0x12345678u32,0xfedcba98u32];
        let blob: [u8;36] = palette_to_blob(&expected);
        let result: Palette = blob_to_palette(&blob);
        for i in 0..8 {
            assert_eq!(result[i],expected[i]);
        }

    }
}


