use std::fs::remove_file;
use thumbnailer::ThumbnailSize;
use thumbnailer::create_thumbnails;
use thumbnailer::error::ThumbResult;
use anyhow::{anyhow,Result};
use std::io::{BufReader};
use std::ffi::OsStr;
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
       Err(err) => Err(anyhow!(err)),
   }
}


pub fn read_or_create_image_data(file_path: &str) -> Result<ImageData> {
    let image_data_file_path = image_data_file_path(file_path);
    match read_image_data(&image_data_file_path) {
        Ok(image_data) => Ok(image_data),
        Err(_) => {
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
                        Err(err) => Err(anyhow!(err)),
                    }
                },
                Err(err) => Err(anyhow!(err)),
            }
        }
    }
}

pub fn delete_file(file_path: &str) {
    let path = Path::new(file_path);
    if path.exists() {
        let _ = remove_file(path);
    }
}

pub fn copy_file_to_target_directory(source_file_path_str: &str, target_directory_name: &str) -> Result<u64> {
    let source_file_path = Path::new(&source_file_path_str);
    let source_file_name = source_file_path.file_name().expect("can't extract file name");
    let target_directory_path = Path::new(&target_directory_name);
    let target_file_path = target_directory_path.join(source_file_name);
    if source_file_path.to_str() != target_file_path.to_str() {
        println!("copy {} to {}", source_file_path.display(), target_file_path.display());
        match std::fs::copy(source_file_path, target_file_path) {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!(err)),
        }
    } else {
        Err(anyhow!("source and target files are identical"))
    }
}

pub fn read_image_data(file_path: &str) -> Result<ImageData> {
    let path = Path::new(file_path);
    if path.exists() {
        match read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(image_data) => Ok(image_data),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    } else {
        Err(anyhow!(format!("image_data {} not found", file_path)))
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
        Err(err) => Err(anyhow!(err)),
    }
}

pub fn get_palette_from_picture(file_path: &str) -> Result<(Palette,Colors)> {
    let image = image::open(file_path).expect(&format!("can't open image file {} for palette extraction", file_path));
    let palette = get_palette(&image);
    let colors = get_colors(&image);
    Ok((palette,colors))
}

fn write_thumbnail<R: std::io::Seek + std::io::Read>(reader: BufReader<R>, extension: &str, mut output_file: File) -> ThumbResult<()> {
    let mime = match extension {
        "jpg" | "jpeg" | "JPG" | "JPEG" => mime::IMAGE_JPEG,
        "png" | "PNG" => mime::IMAGE_PNG,
        _ => panic!("wrong extension"),
    };
    let mut thumbnails = match create_thumbnails(reader, mime, [ThumbnailSize::Small]) {
        Ok(tns) => tns,
        Err(err) => {
            println!("error while creating thumbnails:{:?}", err);
            return Err(err)
        },
    };
    let thumbnail = thumbnails.pop().unwrap();
    let write_result = match extension {
        "jpg" | "jpeg" | "JPG" | "JPEG" => thumbnail.write_jpeg(&mut output_file,255),
        "png" | "PNG" => thumbnail.write_png(&mut output_file),
        _ => panic!("wrong extension"),
    };
    match write_result {
        Err(err) => {
            println!("error while writing thunbnail:{}", err);
            Err(err)
        },
        ok => ok,
    }
}

pub fn check_or_create_thumbnail_file(thumbnail_file_path: &str, original_file_path: &str) -> Result<()> {
    let path = PathBuf::from(thumbnail_file_path);
    if path.exists() {
        Ok(())
    } else {
        println!("creating thumbnail file {}", thumbnail_file_path);
        match File::open(original_file_path) {
            Err(err) => Err(anyhow!(err)),
            Ok(input_file) => {
                let source_path = Path::new(&original_file_path);
                let extension = match source_path.extension()
                    .and_then(OsStr::to_str) {
                        None => return Err(anyhow!("source file has no extension")),
                        Some(ext) => ext,
                    };

                let reader = BufReader::new(input_file);
                let output_file = match File::create(thumbnail_file_path) {
                    Err(err) => return Err(anyhow!(err)),
                    Ok(file) => file,
                };
                match write_thumbnail(reader, extension, output_file) {
                    Err(err) => Err(anyhow!(err)),
                    Ok(_) => Ok(()),
                }
            },
        }
    }
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
    fn read_image_data_deserializes_image_data() {
        let result = read_image_data("testdata/smallIMAGE_DATA.json");
        let expected = ImageData { colors: 15530, rank: Rank::NoStar, selected: false, palette: [2897673, 3959812, 4873222, 7969303, 9277061, 10988432, 12831138, 12896956, 16514043], label: String::from("") };
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
    fn read_picture_file_info_read_file_size() {
        let result = read_file_info("testdata/nature/flower.jpg");
        assert_eq!(true, result.is_ok());
        let (file_size, _modified_time) = result.unwrap();
        assert_eq!(36287, file_size);
    }
}

