use regex::Regex;
use std::path::PathBuf;
use crate::picture_entry::PictureEntry;
use std::collections::HashSet;
use crate::path::get_picture_file_paths;
use crate::args::Args;
use anyhow::{anyhow,Result};
use crate::Database;
use crate::path::check_file;
use crate::picture_entry::{PictureEntries};

pub fn check_database_and_files(directory: &str, database: &Database) -> Result<()> {
    println!("checking database and files");
    match database.select_all_picture_file_paths() {
        Ok(database_file_paths) => {
            match file_paths_in_database_not_on_file_system(&database_file_paths) {
                Ok(file_paths) => {
                    if file_paths.len() > 0 {
                        println!("the following picture file are missing for the images to be shown:");
                        for file_path in file_paths {
                            println!("{}", file_path)
                        }
                    }
                },
                Err(err) => return Err(anyhow!(err)),
            };
            match file_path_in_directory_not_in_database(directory, &database_file_paths) {
                Ok(file_system_file_paths) => {
                    if file_system_file_paths.len() > 0 {
                        println!("the following picture files are not in the database:");
                        for file_path in file_system_file_paths {
                            println!("{}", file_path)
                        }
                    }
                    Ok(())
                },
                Err(err) => return Err(anyhow!(err)),
            }
        },
        Err(err) => return Err(anyhow!(err)),
    }
}

pub fn file_paths_in_database_not_on_file_system(database_file_paths: &Vec<String>) -> Result<Vec<String>> {
    let mut result = vec![];
    for file_path in database_file_paths {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            result.push(file_path.clone())
        }
    };
    Ok(result)
}

pub fn file_path_in_directory_not_in_database(directory: &str, database_file_paths: &Vec<String>) -> Result<Vec<String>> {
    let mut database_set: HashSet<String> = HashSet::new();
    for file_path in database_file_paths {
        let _ = database_set.insert(file_path.clone());
    };
    let mut file_system_set: HashSet<String> = HashSet::new();
    match get_picture_file_paths(directory) {
        Ok(file_paths) => {
            for file_path in file_paths {
                let _ = file_system_set.insert(file_path.to_string());
            }
        },
        Err(err) => return Err(anyhow!(err))
    };
    let difference = file_system_set.difference(&database_set);
    let mut result: Vec<String> = vec![];
    for file_path in difference.collect::<Vec<_>>().iter() {
        result.push(file_path.to_string())
    }
    Ok(result)
}

pub fn load_single_picture_entry(database: &Database, file_path: &str) -> Result<PictureEntries> {
    let mut picture_entries:PictureEntries = vec![];
    match check_file(file_path) {
        Ok(_) => match database.retrieve_or_insert_picture_entry(file_path) {
            Ok(Some(picture_entry)) => {
                picture_entries.push(picture_entry);
                Ok(picture_entries)
            },
            Ok(None) => {
                load_single_picture_entry_from_file_system(file_path)
            },
            Err(err) => Err(anyhow!(err)),
        },
        Err(err) => Err(anyhow!(err)),
    }
}

pub fn load_single_picture_entry_from_file_system(file_path: &str) -> Result<PictureEntries> {
    let mut picture_entries:PictureEntries = vec![];

    match PictureEntry::from_file(file_path) {
        Ok(picture_entry) => {
            picture_entries.push(picture_entry);
            Ok(picture_entries)
        },
        Err(err) => Err(anyhow!(err)),
    }
}
pub fn load_picture_entries_from_covers(database: &mut Database) -> Result<PictureEntries> {
    database.select_cover_picture_entries()
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
        load_single_picture_entry(database, &file)
    } else if args.covers {
        load_picture_entries_from_covers(database)
    } else if args.directory.is_some() && args.add.is_some() && args.from.is_some() {
        load_picture_entries_from_directory_into_db(database, &args.clone().add.unwrap(), true)
    } else if args.directory.is_some() {
        load_picture_entries_from_directory(database, &args.directory.clone().unwrap(), &args)
    } else  {
        load_picture_entries_from_db(database, &args)
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
    match database.select_pictures(&(restriction + &pattern)) {
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

pub fn load_picture_entries_from_directory(database: &mut Database, directory: &str, args: &Args) -> Result<PictureEntries> {
    let args = args.clone();
    match get_picture_file_paths(directory) {
        Ok(file_paths) => {
            let total = file_paths.len();
            let mut count = 0;
            let mut errors = 0;
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
                            errors += 1
                        }
                    }
                };
                count += 1;
            }
            if errors > 0 {
                println!("{} pictures could not be opened", errors);
                Ok(picture_entries)
            } else {
                Ok(picture_entries)
            }
        },
        Err(err) => Err(anyhow!(err)),
    }
}
