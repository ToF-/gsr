use crate::picture_entry::PictureEntry;
use std::collections::HashSet;
use regex::Regex;
use crate::path::get_picture_file_paths;
use crate::args::Args;
use anyhow::{anyhow,Result};
use crate::Database;
use crate::path::check_file;
use crate::picture_entry::{PictureEntries};

pub fn load_picture_entry_from_file_path(database: &Database, file_path: &str) -> Result<PictureEntries> {
    let mut picture_entries:PictureEntries = vec![];
    match check_file(file_path) {
        Ok(_) => match database.retrieve_or_create_image_data(file_path) {
            Ok(Some(picture_entry)) => {
                picture_entries.push(picture_entry);
                Ok(picture_entries)
            },
            Ok(None) => Err(anyhow!("{} image data not found in database'", file_path)),
            Err(err) => Err(anyhow!(err)),
        },
        Err(err) => Err(anyhow!(err)),
    }
}

pub fn load_picture_entries_from_covers(database: &mut Database) -> Result<PictureEntries> {
    database.select_cover_pictures()
}
pub fn load_picture_entries_from_directory_into_db(database: &mut Database, directory: &str, in_std_dir: bool) -> Result<PictureEntries> {
    match database.insert_difference_from_directory(&directory, in_std_dir) {
        Ok(picture_entries) => Ok(picture_entries),
        Err(err) => Err(anyhow!(err)),
    }
}
pub fn load_picture_entries_from_source(database: &mut Database, args: &Args) -> Result<PictureEntries> {
    let args = args.clone();
    if let Some(file) = &args.file {
        load_picture_entry_from_file_path(database, &file)
    } else if args.covers {
        load_picture_entries_from_covers(database)
    } else if args.directory.is_some() && args.add.is_some() && args.from.is_some() {
        load_picture_entries_from_directory_into_db(database, &args.clone().add.unwrap(), true)
    } else if let Some(directory) = &args.directory {
        load_picture_entries_from_directory(database, directory, &args)
    } else {
        Ok(vec![])
    }
}
pub fn load_picture_entries_from_db(database: &mut Database, args: &Args) -> Result<PictureEntries> {
    let args = args.clone();
    let restriction: String = match args.query.clone() {
        Some(s) => s,
        None => String::from("true"),
    };
    let pattern: String = match args.pattern.clone() {
        Some(s) => String::from(" and File_Path like '%".to_owned() + &s + "%'"),
        None => String::from(""),
    };
    match database.select_pictures(restriction + &pattern) {
        Ok(mut picture_entries) => {
            for picture_entry in &mut picture_entries {
                match database.entry_tags(&picture_entry.file_path) {
                    Ok(lags) => {
                        picture_entry.tags = lags
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            };
            Ok(picture_entries)
        },
        Err(err) => return Err(anyhow!(err)),
    }
}
pub fn load_picture_entries_from_db_with_tag_selection(database: &mut Database, args: &Args) -> Result<PictureEntries> {
    println!("loading picture data from database");
    let args = args.clone();
    match &args.select {
        Some(labels) => match database.select_pictures_with_tag_selection(labels.to_vec()) {
            Ok(mut picture_entries) => {
                for picture_entry in &mut picture_entries {
                    match database.entry_tags(&picture_entry.file_path) {
                        Ok(labels) => {
                            picture_entry.tags = labels
                        },
                        Err(err) => return Err(anyhow!(err)),
                    }
                };
                Ok(picture_entries)
            },
            Err(err) => return Err(anyhow!(err)),
        },
        None => load_picture_entries_from_db(database, &args)
    }
}
pub fn load_picture_entries_from_directory(database: &mut Database, directory: &str, args: &Args) -> Result<PictureEntries> {
    println!("loading picture data from directory {}", directory);
    let args = args.clone();
    match get_picture_file_paths(directory) {
        Ok(file_paths) => {
            let mut error: bool = false;
            let mut picture_entries: PictureEntries = vec![];
            for file_path in file_paths {
                let matches_pattern = match args.pattern {
                    None => true,
                    Some(ref pattern) => {
                        match Regex::new(&pattern) {
                            Ok(reg_expr) => match reg_expr.captures(&file_path) {
                                Some(_) => true,
                                None => false,
                            },
                            Err(err) => return Err(anyhow!(err)),
                        }
                    },
                };
                let tag_select_set:HashSet<String> = match args.select {
                    Some(ref tag_list) =>  HashSet::from_iter(tag_list.iter().cloned()),
                    None => HashSet::new(),
                };
                let tag_include_set:HashSet<String> = match args.include {
                    Some(ref tag_list) =>  HashSet::from_iter(tag_list.iter().cloned()),
                    None => HashSet::new(),
                };
                let entry_tags: HashSet<String>;
                if tag_select_set.len() > 0 || tag_include_set.len() > 0 {
                    match database.entry_tags(&file_path) {
                        Ok(tags) => {
                            entry_tags = HashSet::from_iter(tags.iter().cloned())
                        },
                        Err(err) => return Err(anyhow!(err)),
                    }
                } else {
                    entry_tags = HashSet::new()
                }
                let matches_select = match tag_select_set.len() {
                    0 => true,
                    _ => entry_tags.intersection(&tag_select_set).count() > 0,
                };
                let matches_include = match tag_include_set.len() {
                    0 => true,
                    _ => tag_include_set.is_subset(&entry_tags) ,
                };
                if matches_pattern && matches_select && matches_include {
                    match PictureEntry::from_file_or_database(&file_path, database) {
                        Ok(picture_entry) => {
                            picture_entries.push(picture_entry)
                        },
                        Err(err) => {
                            eprintln!("{}", err);
                            error = true;
                        }
                    }
                }
            }
            if error {
                Err(anyhow!(format!("Some pictures could not be opened")))
            } else {
                Ok(picture_entries)
            }
        },
        Err(err) => Err(anyhow!(err)),
    }
}
