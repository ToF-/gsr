use crate::path::get_picture_file_paths;
use rusqlite::Row;
use crate::path::{is_prefix_path, standard_directory,file_path_directory};
use std::collections::HashSet;
use std::collections::HashMap;
use crate::picture_io::read_file_info;
use crate::picture_io::get_palette_from_picture;
use crate::image_data::ImageData;
use crate::picture_entry::make_picture_entry;
use std::io;
use std::time::Duration;
use crate::picture_entry::PictureEntry;
use crate::palette::{palette_to_blob,blob_to_palette};
use rusqlite::{params, Connection};
use std::time::UNIX_EPOCH;
use std::path::Path;
use anyhow::{anyhow, Result};
use std::env;
use crate::Catalog;
use crate::rank::Rank;

const DATABASE_CONNECTION: &str = "GALLSHDB";

#[derive(Debug)]
pub struct Database {
    pub connection_string: String,
    connection: Connection,
}

impl Database {

    pub fn from_path(connection_string: String) -> Result<Self> {
        match Connection::open(connection_string.clone()) {
            Ok(connection) => Ok(Database { connection_string: connection_string.to_string(), connection: connection, }),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn initialize() -> Result<Self> {
        match env::var(DATABASE_CONNECTION) {
            Ok(connection_string) => Self::from_path(connection_string.to_string()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn check_schema(&self) -> Result<()> {
        match self.connection.prepare("SELECT * FROM Picture WHERE rowid = 1;") {
            Ok(_) => match self.connection.prepare("SELECT * FROM Tag WHERE rowid = 1;") {
                    Ok(_) => Ok(()),
                    Err(err) => Err(anyhow!(err)),
                },
            Err(err) => Err(anyhow!(err)),
        }
    }
    // database population policy: if the list of pictures currently in the catalog differs from the
    // list of pictures in the database, one should be asked if any update is in order
    // on confirmation, adding those pictures that are in the catalog, into the database
    // on confirmation, removing those pictures that are in the database and not in the catalog
    // these actions should also be available on command line argument
    //



    pub fn update_database(&self, catalog: &mut Catalog, directory: Option<String>) -> Result<()> {
        match directory {
            Some(dir) => {
                println!("updating database vis à vis directory {}", dir);
                let mut database_set: HashSet<String> = HashSet::new();
                let query_picture = "SELECT file_path from Picture";
                match self.connection.prepare(query_picture) {
                    Ok(mut statement) => {
                        match statement.query_map([], |row| {
                            Ok(row.get::<usize, String>(0).unwrap())
                        }) {
                            Ok(rows) => {
                                for row in rows {
                                    match row {
                                        Ok(file_path) => {
                                            let _ = database_set.insert(file_path);
                                        },
                                        Err(err) => return Err(anyhow!(err)),
                                    }
                                };
                            },
                            Err(err) => return Err(anyhow!(err)),
                        }
                    },
                    Err(err) => return Err(anyhow!(err)),
                };
                match get_picture_file_paths(&dir) {
                    Ok(entries) => {
                        let directory_set: HashSet<String> = HashSet::from_iter(entries.iter().map(|e| e.clone()));
                        let directory_difference = directory_set.difference(&database_set).filter(|s| is_prefix_path(&standard_directory(), &s));
                        if directory_difference.clone().count() > 0 {
                            println!("pictures in this selection that are not in the database:");
                            for x in directory_difference.clone() {
                                println!("{x}");
                            }
                            println!("insert image data for these {} pictures in the database ?", directory_difference.clone().count());
                            let mut response = String::new();
                            let stdin = io::stdin();
                            stdin.read_line(&mut response).expect("can't read from stdin");
                            match response.chars().next() {
                                Some(ch) if ch == 'y' || ch == 'Y' => {
                                    match self.populate(catalog, Some(HashSet::from_iter(directory_difference.into_iter()))) {
                                        Ok(()) => Ok(()),
                                        Err(err) => return Err(anyhow!(err)),
                                    }
                                },
                                Some(_)| None => Ok(()),
                            }
                        } else {
                            Ok(())
                        }
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            }
            None => Ok(())
        }
    }

    pub fn check_create_schema(&self) -> Result<()> {
        println!("checking database {} picture table", self.connection_string);
        let query = "SELECT file_path from Picture";
        match self.connection.prepare(query) {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    fn populate(&self, mut catalog: &mut Catalog, difference_opt: Option<HashSet<&String>>) -> Result<()> {
        let mut count: usize = 0;
        let total = match difference_opt {
            Some(ref difference) => difference.len(),
            None => catalog.length(),
        };
        if total > 0 {
            for file_path in difference_opt.unwrap() {
                let entry = PictureEntry::from_file(file_path);
                    match self.insert_image_data(file_path) {
                        Ok(picture_entry) => match catalog.add_picture_entry_from_file(file_path) {
                            Ok(()) => {},
                            Err(err) => return Err(anyhow!(err)),
                        },
                        Err(err) => return Err(anyhow!(err)),
                    }; 
                    count += 1;
                    println!("{}/{}", count, total);
                };
                Ok(())
            } else {
            Ok(())
        }
    }

    pub fn insert_tag_label(&self, entry: &PictureEntry, label: String) -> Result<()> {
        match self.connection.execute("INSERT INTO Tag VALUES (?1, ?2);", params![&*entry.file_path, label]) {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn delete_tag_label(&self, entry: &PictureEntry, label: String) -> Result<()> {
        match self.connection.execute("DELETE FROM Tag WHERE File_Path = ?1 AND Label = ?2;", params![&*entry.file_path, label]) {
            Ok(count) => {
                println!("{} row deleted", count);
                Ok(())
            },
            Err(err) => {
                eprintln!("{}", err);
                Err(anyhow!(err))
            },
        }
    }
    pub fn update_image_data(&self, entry: &PictureEntry) -> Result<()> {
        match self.connection.execute(" UPDATE Picture SET File_Size = ?1, Colors = ?2, Modified_Time = ?3, Rank = ?4, Palette = ?5, Label = ?6, Selected = ?7, Deleted = ?8 WHERE File_Path = ?9;",
        params![entry.file_size as i64,
        entry.colors as i64,
        entry.modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
        entry.rank as i64,
        palette_to_blob(&entry.palette),
        if entry.label().is_some() { entry.label().unwrap() } else { String::from("") },
        entry.selected as i64,
        entry.selected as i64,
        &*entry.file_path]) {
            Ok(updated) => {
                println!("{} row updated", updated);
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    fn sql_to_picture_entry(row: &Row) -> Result<PictureEntry> {
        Ok(make_picture_entry(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                {
                    let mt:i64 = row.get(3)?;
                    UNIX_EPOCH + Duration::new(mt as u64, 0)
                },
                {
                    let r:i64 = row.get(4)?;
                    Rank::from(r)
                },
                {
                    let blob: Vec<u8> = row.get(5)?;
                    let mut bytes: [u8;36] = [0;36];
                    for i in 0..36 { bytes[i] = blob[i] };
                    Some(blob_to_palette(&bytes))
                },
                {
                    let label:String = match row.get(6) {
                        Ok(s) => s,
                        _ => String::from(""),
                    };
                    if label.trim().len() > 0 {
                        Some(label.trim().to_string())
                    } else {
                        None
                    }
                },
                {
                    let result:bool = row.get(7)?;
                    result
                },
                {
                    let result:bool = row.get(8)?;
                    result
                },
                vec![],))
    }

    pub fn delete_picture(&self, file_path: String) -> Result<()> {
        match self.connection.execute("DELETE FROM Picture WHERE File_Path = ?1;", params![file_path.clone()]) {
            Ok(count) => {
                println!("{} picture deleted", count);
                match self.connection.execute("DELETE FROM Tag WHERE File_Path = ?1;", params![file_path.clone()]) {
                    Ok(count) => {
                        println!("{} tags deleted", count);
                        Ok(())
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn select_pictures(&self, query: String) -> Result<Vec<PictureEntry>> {
        match self.connection.prepare(&("SELECT File_Path, File_Size, Colors, Modified_Time, Rank, Palette, Label, Selected, Deleted FROM Picture WHERE ".to_owned() + &query + ";")) {
            Ok(mut statement) => {
                match statement.query([]) {
                    Ok(mut rows) => {
                        let mut result: Vec<PictureEntry> = vec![];
                        while let Some(row) = rows.next()? {
                            match Self::sql_to_picture_entry(row) {
                                Ok(entry) => {
                                    result.push(entry);
                                },
                                Err(err) => return Err(anyhow!(err)),
                            }
                        };
                        return Ok(result)
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn select_pictures_with_tag_selection(&self, select:Vec<String>) -> Result<Vec<PictureEntry>> {
        let tag_labels: String = select.into_iter().map(|s| format!("'{}'", s)).collect::<Vec<String>>().join(",");
        match self.connection.prepare(&("SELECT DISTINCT(Picture.File_Path), File_Size, Colors, Modified_Time, Rank, Palette, Picture.Label, Selected, Deleted FROM Picture INNER JOIN Tag ON Tag.File_Path = Picture.File_Path WHERE Tag.Label IN (".to_owned() + &tag_labels + ");")) {
            Ok(mut statement) => {
                match statement.query([]) {
                    Ok(mut rows) => {
                        let mut result: Vec<PictureEntry> = vec![];
                        while let Some(row) = rows.next()? {
                            match Self::sql_to_picture_entry(row) {
                                Ok(entry) => {
                                    result.push(entry);
                                },
                                Err(err) => return Err(anyhow!(err)),
                            }
                        };
                        return Ok(result)
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

pub fn load_all_tags(&self) -> Result<Vec<String>> {
    let mut result: Vec<String> = vec![];
    let query = "SELECT DISTINCT Label FROM Tag;";
    match self.connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query_map([], |row| {
                Ok(row.get::<usize, String>(0).unwrap())
                }) {
                    Ok(rows) => {
                        for row in rows {
                            match row {
                                Ok(label) => {
                                        result.push(label);
                                },
                                Err(err) => return Err(anyhow!(err)),
                            }
                        };
                        Ok(result)
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            },
            Err(err) => return Err(anyhow!(err)),
        }
    }

    pub fn load_directories(&self) -> Result<Vec<(String,usize)>> {
        let mut dir_map: HashMap<String,usize> = HashMap::new();
        let query = "SELECT File_Path from Picture;";
        match self.connection.prepare(query) {
            Ok(mut statement) => {
                match statement.query_map([], |row| {
                    Ok(row.get::<usize, String>(0).unwrap())
                }) {
                    Ok(rows) => {
                        for row in rows {
                            match row {
                                Ok(file_path) => {
                                    let directory =file_path_directory(&file_path);
                                    dir_map.entry(directory).and_modify(|files| *files += 1).or_insert(1);
                                },
                                Err(err) => return Err(anyhow!(err)),
                            }
                        };
                        let mut result:Vec<(String,usize)> = Vec::from_iter(dir_map.iter().map(|pair| (pair.0.clone(), pair.1.clone())));
                        result.sort();
                        Ok(result)
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }
    pub fn entry_tags(&self, file_path: &str) -> Result<Vec<String>> {
        let mut result: Vec<String> = vec![];
        let query = "SELECT DISTINCT Label FROM Tag WHERE File_Path = ?1;";
        match self.connection.prepare(query) {
            Ok(mut statement) => {
                match statement.query_map([file_path], |row| {
                    Ok(row.get::<usize, String>(0).unwrap())
                }) {
                    Ok(rows) => {
                        for row in rows {
                            match row {
                                Ok(label) => {
                                        result.push(label);
                                },
                                Err(err) => return Err(anyhow!(err)),
                            }
                        };
                        Ok(result)
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            },
            Err(err) => return Err(anyhow!(err)),
        }
    }

    pub fn insert_image_data(&self, file_path: &str) -> Result<PictureEntry> {
        match read_file_info(file_path) {
            Ok((file_size, modified_time)) => {
                match get_palette_from_picture(file_path) {
                    Ok((palette, colors)) => {
                        let image_data = ImageData{
                            colors: colors,
                            rank: Rank::NoStar,
                            selected: false,
                            palette: palette,
                            label: String::from(""),
                        };
                        let entry = make_picture_entry(file_path.to_string(), file_size, image_data.colors, modified_time, image_data.rank, Some(image_data.palette), Some(image_data.label), image_data.selected, false, vec![]);
                        match self.connection.execute(" INSERT INTO Picture 
            (File_path, File_size, Colors, Modified_time, Rank, Label, Selected, Deleted, Palette)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
            params![
            entry.file_path,
            entry.file_size as i64,
            entry.colors as i64,
            entry.modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            entry.rank as i64,
            entry.label(),
            entry.selected as i64,
            entry.selected as i64,
            palette_to_blob(&entry.palette)]) {
                            Ok(inserted) => {
                                println!("{} row inserted", inserted);
                                return Ok(entry)
                            },
                            Err(err) => return Err(anyhow!(err)),
                        }
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn retrieve_or_create_image_data(&self, file_path: &str) -> Result<Option<PictureEntry>> {
        match self.connection.prepare(" SELECT File_Path, File_Size, Colors, Modified_Time, Rank, Palette, Label, Selected, Deleted, rowid FROM Picture WHERE File_Path = ?1;") {
            Ok(mut statement) => match statement.query([file_path]) {
                Ok(mut rows) => match rows.next() {
                    Ok(Some(row)) => match Self::sql_to_picture_entry(row) {
                        Ok(mut entry) => match self.entry_tags(&entry.file_path) {
                            Ok(labels) => {
                                entry.tags = labels;
                                Ok(Some(entry))
                            },
                            Err(err) => Err(anyhow!(err)),
                        },
                        Err(err) => Err(anyhow!(err)),
                    },
                    Ok(None) => {
                        if standard_directory() != "" && file_path_directory(file_path) == standard_directory() {
                            println!("couldn't find {} in database; insert image data from this file ?", file_path);
                            let mut response = String::new();
                            let stdin = io::stdin();
                            stdin.read_line(&mut response).expect("can't read from stdin");
                            match response.chars().next() {
                                Some(ch) if ch == 'y' || ch == 'Y' => {
                                    match self.insert_image_data(file_path) {
                                        Ok(picture_entry) => Ok(Some(picture_entry)),
                                        Err(err) => return Err(anyhow!(err)),
                                    }
                                },
                                Some(_)|
                                    None => Ok(None),
                            }
                        } else {
                            Ok(None)
                        }
                    },
                    Err(err) => Err(anyhow!(err)),
                },
                Err(err) => Err(anyhow!(err)),
            },
            Err(err) => Err(anyhow!(err)),
        }
    }
}

