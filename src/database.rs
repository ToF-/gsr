use std::path::{Path,PathBuf};
use crate::path::get_picture_file_paths;
use rusqlite::{Row, Error};
use crate::path::{is_prefix_path, standard_directory,file_path_directory};
use std::collections::HashSet;
use std::collections::HashMap;
use crate::picture_io::read_file_info;
use crate::picture_io::get_palette_from_picture;
use crate::image_data::ImageData;
use crate::picture_entry::make_picture_entry;
use std::io;
use std::time::Duration;
use crate::picture_entry::{PictureEntry, PictureEntries};
use crate::palette::{palette_to_blob,blob_to_palette};
use rusqlite::{params, Connection};
use std::time::UNIX_EPOCH;
use anyhow::{anyhow, Result};
use std::env;
use crate::rank::Rank;

const DATABASE_CONNECTION: &str = "GALLSHDB";

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

impl Database {

    pub fn from_path(connection_string: &str) -> Result<Self> {
        match Connection::open(connection_string) {
            Ok(connection) => Ok(Database { connection: connection, }),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn initialize(create_schema: bool) -> Result<Self> {
        match env::var(DATABASE_CONNECTION) {
            Ok(connection_string) => match Self::from_path(&connection_string) {
                Ok(database) => {
                    if create_schema {
                        match database.create_schema() {
                            Ok(()) => {},
                            Err(err) => return Err(anyhow!(err)),
                        }
                    };
                    println!("database: {}", connection_string);
                    Ok(database)
                },
                Err(err) => Err(anyhow!(err)),
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn create_schema(&self) -> Result<()> {
        println!("creating database schema");
        match self.connection.execute(
            "CREATE TABLE IF NOT EXISTS Picture (\n\
                File_Path TEXT NOT NULL PRIMARY KEY,\n\
                File_Size INTEGER,\n\
                Colors INTEGER,\n\
                Modified_Time INTEGER,\n\
                Rank INTEGER,\n\
                Palette BLOB,\n\
                Label TEXT,\n\
                Selected BOOLEAN,\n\
                Deleted BOOLEAN);", []) {
            Ok(_) => match self.connection.execute(
                "CREATE TABLE IF NOT EXISTS Tag (\n\
                    File_Path TEXT NOT NULL,\n\
                    Label TEXT NOT NULL,\n\
                    PRIMARY KEY ( File_Path, Label));", []) {
                Ok(_) => match self.connection.cache_flush() {
                    Ok(_) => Ok(()),
                    Err(err) => Err(anyhow!(err)),
                }
                Err(err) => Err(anyhow!(err)),
            },
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

    fn decimate(&self, file_paths_set: HashSet<String>) -> Result<HashSet<String>> {
        let mut total_tags: usize = 0;
        let mut total_pics: usize = 0;
        for file_path in file_paths_set.iter() {
            match self.connection.execute("DELETE FROM Tag WHERE File_Path = ?1;", params![file_path]) {
                Ok(count) => { total_tags += count; },
                Err(err) => return Err(anyhow!(err)),
            };
            match self.connection.execute("DELETE FROM Picture WHERE File_Path = ?1", params![file_path]) {
                Ok(count) => { total_pics += count; },
                Err(err) => return Err(anyhow!(err)),
            };
        };
        println!("{} pictures, {} tags deleted", total_pics, total_tags);
        Ok(file_paths_set)
    }

    fn populate(&self, difference_opt: Option<HashSet<&String>>) -> Result<PictureEntries> {
        let mut count: usize = 0;
        let total = match difference_opt {
            Some(ref difference) => difference.len(),
            None => 0,
        };
        if total > 0 {
            let mut picture_entries: PictureEntries = vec![];
            for file_path in difference_opt.unwrap() {
                match self.insert_image_data(file_path) {
                    Ok(entry) => {
                        picture_entries.push(entry)
                    },
                    Err(err) => return Err(anyhow!(err)),
                }; 
                count += 1;
                println!("{}/{}", count, total);
            };
            Ok(picture_entries)
        } else {
            Ok(vec![])
        }
    }

    pub fn delete_cover(&self, dir_path: &str, file_name: &str) -> Result<()> {
        match self.connection.execute("DELETE FROM Cover WHERE Dir_path = ?1 AND File_Name = ?2;",
            params![dir_path, file_name]) {
            Ok(deleted) => {
                println!("{}", "⌫".repeat(deleted));
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn insert_cover(&self, dir_path: &str, file_name: &str, rank: Rank) -> Result<()> {
        match self.connection.execute("INSERT INTO Cover VALUES (?1, ?2, ?3);", 
            params![dir_path, file_name, rank as i64]) {
            Ok(inserted) => {
                println!("{}", "⎀".repeat(inserted));
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn insert_tag_label(&self, entry: &PictureEntry, label: &str) -> Result<()> {
        match self.connection.execute("INSERT INTO Tag VALUES (?1, ?2);", params![&*entry.file_path, label]) {
            Ok(inserted) => {
                println!("{}", "⎀".repeat(inserted));
                Ok(())
            },
            Err(err) => Err(anyhow!(err)),
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
                println!("{}", "⟳".repeat(updated));
                match self.connection.execute(" DELETE FROM Tag WHERE File_Path = ?1;", params![&*entry.file_path]) {
                    Ok(deleted) => {
                        println!("{}", "⌫".repeat(deleted));
                        for tag in entry.tags.iter() {
                            match self.insert_tag_label(entry, tag) {
                                Ok(()) => {},
                                Err(err) => return Err(anyhow!(err)),
                            }
                        };
                        Ok(())
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
            }
    }

    pub fn select_cover_pictures(&mut self) -> Result<PictureEntries> {
        match self.connection.prepare("SELECT File_Path, File_Size, Colors, Modified_Time, P.Rank, Palette, Label, Selected, Deleted FROM Picture P JOIN Cover C ON P.File_Path = CONCAT(C.Dir_Path, '/', C.File_Name);") {
            Ok(mut statement) => {
                match statement.query([]) {
                    Ok(mut rows) => {
                        let mut result: PictureEntries = vec![];
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

    fn sql_to_picture_entry(row: &Row) -> Result<PictureEntry> {
        match Self::rusqlite_to_picture_entry(row) {
            Ok(picture_entry) => Ok(picture_entry),
            Err(err) => Err(anyhow!(err)),
        }
    }

    fn rusqlite_to_picture_entry(row: &Row) -> Result<PictureEntry,Error> {
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
                HashSet::new(),))
    }

    fn rusqlite_select_all_picture_file_paths(&self) -> Result<Vec<String>,Error> {
        self.connection.prepare("SELECT File_Path FROM Picture;")
            .and_then(|mut statement| { statement.query([])
                .and_then(|mut rows| {
                    let mut result:Vec<String> = vec![];
                    while let Some(row) = rows.next()? {
                        let file_path:String = row.get(0)?;
                        result.push(file_path.clone())
                    };
                    Ok(result)
                })
            })
    }

    pub fn select_all_picture_file_paths(&self) -> Result<Vec<String>> {
        match self.rusqlite_select_all_picture_file_paths() {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!(err)),
        }
    }

    fn rusqlite_delete_picture(&self, file_path: &str) -> Result<(),Error> {
        self.connection.execute(
            "DELETE FROM Picture \n\
             WHERE File_Path = ?1;", params![file_path.to_string()])
            .and_then(|_| {
                self.connection.execute(
                    "DELETE FROM Tak  \n\
                     WHERE File_Path = ?1;", params![file_path.to_string()])
                    .and_then(|_| Ok(()))
            })
    }

    pub fn delete_picture(&self, file_path: &str) -> Result<()> {
        match self.rusqlite_delete_picture(file_path) {
            Ok(()) => Ok(()),
            Err(err) => return Err(anyhow!(err)),
        }
    }

    fn rusqlite_select_pictures(&self, query: String) -> Result<PictureEntries, Error> {
            self.connection.prepare(
                &("SELECT File_Path,     \n\
                          File_Size,     \n\
                          Colors,        \n\
                          Modified_Time, \n\
                          Rank,          \n\
                          Palette,       \n\
                          Label,         \n\
                          Selected,      \n\
                          Deleted        \n\
                          FROM Picture   \n\
                          WHERE ".to_owned() + &query + ";"))
                .and_then( |mut statement| { statement.query([])
                    .and_then( |mut rows| {
                        let mut picture_entries: PictureEntries = vec![];
                        while let Some(row) = rows.next()? {
                            match Self::rusqlite_to_picture_entry(row) {
                                Ok(picture_entry) => { picture_entries.push(picture_entry); },
                                Err(err) => return Err(err),
                            }
                        };
                        Ok(picture_entries) })
                })
    }

    pub fn select_pictures(&self, query: String) -> Result<PictureEntries> {
        match self.rusqlite_select_pictures(query) {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn delete_difference_from_file_system(&mut self) -> Result<HashSet<String>> {
        match self.connection.prepare("SELECT file_path from Picture;") {
            Ok(mut statement) => match statement.query_map([], |row| Ok(row.get::<usize,String>(0).unwrap())) {
                Ok(rows) => {
                    let mut difference: HashSet<String> = HashSet::new();
                    for row in rows {
                        match row {
                            Ok(file_path) => {
                                let path = PathBuf::from(file_path.clone());
                                if ! path.exists() { let _ = difference.insert(file_path); } else { };
                            },
                            Err(err) => return Err(anyhow!(err)),
                        }
                    };
                    if difference.len() > 0 {
                        println!("this pictures from the database are no longer in the file system:");
                        for x in difference.clone() {
                            println!("{x}");
                        }
                        println!("delete image data for these {} pictures from the database ?", difference.len());
                        let mut response = String::new();
                        let stdin = io::stdin();
                        stdin.read_line(&mut response).expect("can't read from stdin");
                        match response.chars().next() {
                            Some(ch) if ch == 'y' || ch == 'Y' => {
                                self.decimate(difference)
                            },
                            Some(_)| None => Ok(HashSet::new()),
                        }
                    } else { 
                        Ok(HashSet::new())
                    }
                },
                Err(err) => Err(anyhow!(err)),
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn insert_difference_from_directory(&mut self, directory: &str, in_std_dir:bool) -> Result<PictureEntries> {
        let path = Path::new(directory);
        if path.has_root() {
            match get_picture_file_paths(directory) {
                Ok(file_paths) => {
                    let directory_set: HashSet<String> = HashSet::from_iter(file_paths.iter().map(String::clone));
                    let mut database_set: HashSet<String> = HashSet::new();
                    let query = "SELECT file_path from Picture;";
                    match self.connection.prepare(query) {
                        Ok(mut statement) => match statement.query_map([], |row| Ok(row.get(0).unwrap())) {
                            Ok(rows) => {
                                for row in rows {
                                    match row {
                                        Ok(file_path) => {
                                            let _ = database_set.insert(file_path);
                                        },
                                        Err(err) => return Err(anyhow!(err)),
                                    }
                                };
                                let difference = directory_set.difference(&database_set).filter(|s| !in_std_dir || is_prefix_path(&standard_directory(), &s));
                                if difference.clone().count() > 0 {
                                    println!("pictures in this selection that are not in the database:");
                                    for x in difference.clone() {
                                        println!("{x}");
                                    }
                                    println!("insert image data for these {} pictures in the database ?", difference.clone().count());
                                    let mut response = String::new();
                                    let stdin = io::stdin();
                                    stdin.read_line(&mut response).expect("can't read from stdin");
                                    match response.chars().next() {
                                        Some(ch) if ch == 'y' || ch == 'Y' => {
                                            match self.populate(Some(HashSet::from_iter(difference.into_iter()))) {
                                                Ok(picture_entries) => Ok(picture_entries),
                                                Err(err) => return Err(anyhow!(err)),
                                            }
                                        },
                                        Some(_)| None => Ok(vec![]),
                                    }
                                } else {
                                    Ok(vec![])
                                }
                            },
                            Err(err) => return Err(anyhow!(err)),
                        },
                        Err(err) => return Err(anyhow!(err)),
                    }
                },
                Err(err) => Err(anyhow!(err)),
            }
        } else {
            Err(anyhow!(format!("the path {} is relative and cannot be used as a picture file path", path.display())))
        }
    }

pub fn select_pictures_with_tag_selection(&self, select:Vec<String>) -> Result<PictureEntries> {
    let tag_labels: String = select.into_iter().map(|s| format!("'{}'", s)).collect::<Vec<String>>().join(",");
    match self.connection.prepare(&("SELECT DISTINCT(Picture.File_Path), File_Size, Colors, Modified_Time, Rank, Palette, Picture.Label, Selected, Deleted FROM Picture INNER JOIN Tag ON Tag.File_Path = Picture.File_Path WHERE Tag.Label IN (".to_owned() + &tag_labels + ");")) {
        Ok(mut statement) => {
            match statement.query([]) {
                Ok(mut rows) => {
                    let mut result: PictureEntries = vec![];
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
pub fn entry_tags(&self, file_path: &str) -> Result<HashSet<String>> {
    let mut result: HashSet<String> = HashSet::new();
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
                                result.insert(label);
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
                    let entry = make_picture_entry(file_path.to_string(), file_size, image_data.colors, modified_time, image_data.rank, Some(image_data.palette), Some(image_data.label), image_data.selected, false, HashSet::new());
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
                            println!("{}","⎀".repeat(inserted));
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

