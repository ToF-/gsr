use crate::image_data::ImageData;
use anyhow::{anyhow, Result};
use crate::palette::{palette_to_blob,blob_to_palette};
use crate::path::file_name;
use crate::path::get_picture_file_paths;
use crate::path::{is_prefix_path, standard_directory,file_path_directory};
use crate::picture_entry::make_picture_entry;
use crate::picture_entry::{PictureEntry, PictureEntries};
use crate::prompt::prompt_yes_no;
use crate::rank::Rank;
use rusqlite::{Row, Error};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::path::{Path,PathBuf};
use std::time::Duration;
use std::time::UNIX_EPOCH;

const DATABASE_CONNECTION: &str = "GALLSHDB";

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

impl Database {

    /// initialize the database, creating the schema if needed.
    pub fn initialize(create_schema: bool) -> Result<Self> {
        match env::var(DATABASE_CONNECTION) {
            Ok(connection_string) => match Self::from_path(&connection_string) {
                Ok(database) => {
                    if create_schema {
                        match database.rusqlite_create_schema() {
                            Ok(()) => {},
                            Err(err) => return Err(anyhow!(err)),
                        }
                    };
                    Ok(database)
                },
                Err(err) => Err(anyhow!(err)),
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

    /// update a picture entry in the database
    pub fn update_picture_entry(&mut self, entry: &PictureEntry) -> Result<()> {
        match self.rusqlite_update_image_data(entry) {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    /// insert the cover picture in the database or update it with rank if already existing
    pub fn insert_or_update_cover(&mut self, dir_path: &str, file_name: &str, rank: Rank) -> Result<()> {
        match self.rusqlite_insert_or_update_cover(dir_path, file_name, rank) {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    /// delete the cover picture from the database
    pub fn delete_cover(&mut self, dir_path: &str, file_name: &str) -> Result<()> {
        match self.rusqlite_delete_cover(dir_path, file_name) {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    /// select the picture entries that are covers for their directory
    pub fn select_cover_picture_entries(&mut self) -> Result<PictureEntries> {
        match self.rusqlite_select_cover_picture_entries() {
            Ok(picture_entries) => Ok(picture_entries),
            Err(err) => Err(anyhow!(err)),
        }
    }
    
    /// create the database from the given connection_string.
    fn from_path(connection_string: &str) -> Result<Self> {
        match Connection::open(connection_string) {
            Ok(connection) => Ok(Database { connection }),
            Err(err) => Err(anyhow!(err)),
        }
    }

    /// selects all the pictures entries used as cover for a directory
    fn rusqlite_select_cover_picture_entries(&mut self) -> Result<PictureEntries, Error> {
        self.connection.prepare(
            "SELECT File_Path,            \n\
            File_Size,                    \n\
            Colors,                       \n\
            Modified_Time,                \n\
            Rank, Palette,                \n\
            Label,                        \n\
            Selected,                     \n\
            Deleted,                      \n\
            Cover                         \n\
            FROM Picture                  \n\
            WHERE Cover = True;")
            .and_then(|mut statement| {
                statement.query([])
                    .and_then(|mut rows| {
                        let mut picture_entries = vec![];
                        while let Some(row) = rows.next()? {
                            match Self::rusqlite_to_picture_entry(row) {
                                Ok(picture_entry) => {
                                    picture_entries.push(picture_entry);
                                },
                                Err(err) => return Err(err),
                            }
                        };
                        Ok(picture_entries)
                    })
            })
    }

    /// create the database schema
    fn rusqlite_create_schema(&self) -> Result<(),Error> {
        println!("creating database schema");
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS Picture (    \n\
                File_Path TEXT NOT NULL PRIMARY KEY, \n\
                File_Size INTEGER,                   \n\
                Colors INTEGER,                      \n\
                Modified_Time INTEGER,               \n\
                Rank INTEGER,                        \n\
                Palette BLOB,                        \n\
                Label TEXT,                          \n\
                Selected BOOLEAN,                    \n\
                Deleted BOOLEAN,                     \n\
                Cover BOOLEAN);", [])
            .and_then(|_| {
                self.connection.execute(
                    "CREATE TABLE IF NOT EXISTS Tag ( \n\
                    File_Path TEXT NOT NULL,          \n\
                    Label TEXT NOT NULL,              \n\
                    PRIMARY KEY ( File_Path, Label));", [])
                    .and_then(|_| {
                        self.connection.execute(
                        "CREATE TABLE IF NOT EXISTS Cover (  \n\
                         Dir_Path TEXT NOT NULL,             \n\
                         File_Name TEXT NOT NULL,            \n\
                         Rank INTEGER,                       \n\
                         PRIMARY KEY (Dir_Path, File_Name));", [])
                            .map(|_|  ())
                    })
            })
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
                match self.insert_picture_entry(file_path) {
                    Ok(picture_entry) => {
                        picture_entries.push(picture_entry)
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

    fn rusqlite_delete_cover(&self, dir_path: &str, file_name: &str) -> Result<usize,Error> {
        self.connection.execute("DELETE FROM Cover WHERE Dir_path = ?1 AND File_Name = ?2;",
            params![dir_path, file_name])
    }

    fn rusqlite_insert_cover(&self, dir_path: &str, file_name: &str, rank: Rank) -> Result<usize,Error> {
        self.connection.execute(
            "INSERT INTO Cover            \n\
             (Dir_Path, File_Name, Rank) \n\
             VALUES (?1, ?2, ?3);", 
            params![dir_path, file_name, rank as i64])
    }

    fn rusqlite_insert_or_update_cover(&mut self, dir_path: &str, file_name: &str, rank: Rank) -> Result<(),Error> {
        self.rusqlite_delete_cover(dir_path, file_name)
            .and_then(|_| {
                self.rusqlite_insert_cover(dir_path, file_name, rank)
                    .map(|_| ())
            })
    }

    fn rusqlite_delete_tags_for_file_path(&mut self, file_path: &str) -> Result<(),Error> {
        self.connection.execute(
            "DELETE FROM Tag WHERE File_Path = ?1;",
            params![file_path])
            .map(|_| ())
    }

    fn rusqlite_insert_tag_label(&mut self, file_path: &str, label: &str) -> Result<(),Error> {
        self.connection.execute(
            "INSERT INTO Tag      \n\
            (File_Path, Label)    \n\
            VALUES (?1, ?2);",
            params![file_path, label])
            .map(|_| ())
    }

    fn rusqlite_update_image_data(&mut self, entry: &PictureEntry) -> Result<(),Error> {
        self.connection.execute(
            "UPDATE PICTURE SET         \n\
             File_Size = ?1,            \n\
             Colors = ?2,               \n\
             Modified_Time = ?3,        \n\
             Rank = ?4,                 \n\
             Palette = ?5,              \n\
             Label = ?6,                \n\
             Selected = ?7,             \n\
             Deleted = ?8,              \n\
             Cover = ?9
             WHERE File_Path = ?10;",
             params![
             entry.file_size as i64,
             entry.image_data.colors as i64,
             entry.modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
             entry.image_data.rank as i64,
             palette_to_blob(&entry.image_data.palette),
             if entry.label().is_some() { entry.label().unwrap() } else { String::from("") },
             entry.image_data.selected as i64,
             entry.deleted as i64,
             entry.image_data.cover,
             &*entry.file_path])
                 .and_then(|_| {
                     self.rusqlite_delete_tags_for_file_path(&entry.file_path)
                         .and_then(|_| {
                             for tag in entry.image_data.tags.iter() {
                                 match self.rusqlite_insert_tag_label(&entry.file_path, tag) {
                                     Ok(()) => {},
                                     Err(err) => return Err(err)
                                 }
                             };
                             Ok(())
                         })
             })
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
                {
                    let mt:i64 = row.get(3)?;
                    UNIX_EPOCH + Duration::new(mt as u64, 0)
                },
                ImageData {
                    colors: row.get(2)?,
                    rank: {
                        let r:i64 = row.get(4)?;
                        Rank::from(r)
                    },
                    palette: {
                        let blob: Vec<u8> = row.get(5)?;
                        let mut bytes: [u8;36] = [0;36];
                        bytes.copy_from_slice(&blob[..36]);
                        blob_to_palette(&bytes)
                    },
                    label: {
                        let label:String = row.get(6).unwrap_or_default();
                        if !label.trim().is_empty() {
                            label.trim().to_string()
                        } else {
                            String::new()
                        }
                    },
                    selected: {
                        let result:bool = row.get(7)?;
                        result
                    },
                    cover: {
                        let result:bool = row.get(9)?;
                        result
                    },
                    tags: HashSet::new(),
                },
                {
                    let result:bool = row.get(8)?;
                    result
                },
                ))
    }

    fn rusqlite_select_all_picture_file_paths(&self) -> Result<HashSet<String>,Error> {
        self.connection.prepare("SELECT File_Path FROM Picture;")
            .and_then(|mut statement| { statement.query([])
                .and_then(|mut rows| {
                    let mut result:HashSet<String> = HashSet::new();
                    while let Some(row) = rows.next()? {
                        let file_path:String = row.get(0)?;
                        let _ = result.insert(file_path.clone());
                    };
                    Ok(result)
                })
            })
    }

    pub fn select_all_picture_file_paths(&self) -> Result<HashSet<String>> {
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
                    "DELETE FROM Tag  \n\
                     WHERE File_Path = ?1;", params![file_path.to_string()])
                    .map(|_| ())
            })
    }

    pub fn delete_picture(&self, file_path: &str) -> Result<()> {
        match self.rusqlite_delete_picture(file_path) {
            Ok(()) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }
    pub fn delete_picture_data_where_file_do_not_exists(&mut self) -> Result<usize> {
        let mut count = 0;
        let result = self.rusqlite_select_all_picture_file_paths()
            .and_then(|file_paths| {
                for file_path in file_paths {
                    let path = PathBuf::from(&file_path);
                    let _ = if !path.exists() {
                        match self.rusqlite_delete_picture(&file_path) {
                            Ok(_) => {
                                let _ = self.rusqlite_delete_tags_for_file_path(&file_path)
                                    .and_then(|_| {
                                        let directory = file_path_directory(&file_path);
                                        let file_name = file_name(&file_path);
                                        self.rusqlite_delete_cover(&directory, &file_name)
                                            .map(|_| ())
                                    });
                                count += 1;
                            },
                            Err(e) => return Err(e),
                        }
                    };
                };
                Ok(count)
            });
        match result {
            Ok(count) => Ok (count),
            Err(err) => Err(anyhow!(err)),
        }
    }

    fn rusqlite_select_pictures(&self, query: &str) -> Result<PictureEntries, Error> {
        let full_query: String =
            "SELECT File_Path,     \n\
              File_Size,             \n\
              Colors,                \n\
              Modified_Time,         \n\
              Rank,                  \n\
              Palette,               \n\
              Label,                 \n\
              Selected,              \n\
              Deleted,               \n\
              Cover                  \n\
              FROM Picture           \n\
              WHERE ".to_owned() + query + ";";
        self.connection.prepare(&full_query)
            .and_then(|mut statement| {
                statement.query([])
                    .and_then(|mut rows| {
                            let mut picture_entries: PictureEntries = vec![];
                            while let Some(row) = rows.next()? {
                                match Self::rusqlite_to_picture_entry(row) {
                                    Ok(picture_entry) => { picture_entries.push(picture_entry); },
                                    Err(err) => return Err(err),
                                }
                            };
                            Ok(picture_entries)
                        })
            })
    }

    pub fn select_pictures(&self, query: &str) -> Result<PictureEntries> {
        match self.rusqlite_select_pictures(query) {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn insert_difference_from_directory(&mut self, directory: &str, in_std_dir:bool) -> Result<PictureEntries> {
        println!("insert_difference_from_directory {} {}", directory, in_std_dir);
        let path = Path::new(directory);
        if path.has_root() {
            get_picture_file_paths(directory)
                .and_then(|file_paths| {
                    println!("{:?}", file_paths);
                    let directory_set: HashSet<String> = HashSet::from_iter(file_paths.iter().map(String::clone));
                    self.select_all_picture_file_paths()
                        .and_then(|database_set| {
                            println!("database_set.count:{}", database_set.len());
                            let difference = directory_set.difference(&database_set)
                                .filter(|s| !in_std_dir || is_prefix_path(&standard_directory(), s));
                            if difference.clone().count() > 0 {
                                println!("pictures in this selection that are not in the database:");
                                for x in difference.clone() {
                                    println!("{x}");
                                }
                                match prompt_yes_no(&format!("insert image data for these {} pictures in the database ?", difference.clone().count())) {
                                    Ok(Some('y')) | Ok(Some('Y')) => {
                                        match self.populate(Some(HashSet::from_iter(difference))) {
                                            Ok(picture_entries) => Ok(picture_entries),
                                            Err(err) => Err(anyhow!(err)),
                                        }
                                    },
                                    Ok(_) => Ok(vec![]),
                                    Err(err) => Err(anyhow!(err)),
                                }
                            } else {
                                Ok(vec![])
                            }
                        })
                })
        } else {
            Err(anyhow!(format!("the path {} is relative and cannot be used as a picture file path", path.display())))
        }
    }

    fn rusqlite_load_all_tags(&self) -> Result<HashSet<String>,Error> {
        self.connection.prepare("SELECT DISTINCT Label FROM Tag UNION SELECT DISTINCT Label FROM Picture WHERE Label IS NOT NULL;")
            .and_then(|mut statement| {
                statement.query_map([], |row| {
                    match row.get::<usize, String>(0) {
                        Ok(s) => Ok(s),
                        Err(err) => {
                            eprintln!("{}", err);
                            Ok(String::from(""))
                        }
                    }
                })
                .map(|rows| {
                    let mut result: HashSet<String> = HashSet::new();
                    for row in rows {
                        let label = row.unwrap();
                        if !label.is_empty() {
                            let _ = result.insert(label);
                        };
                    };
                    result
                })
            })
    }

    pub fn load_all_tags(&self) -> Result<HashSet<String>> {
        match self.rusqlite_load_all_tags() {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!(err)),
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
                    let mut result:Vec<(String,usize)> = Vec::from_iter(dir_map.iter().map(|pair| (pair.0.clone(), *pair.1)));
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
                Err(err) => Err(anyhow!(err)),
            }
        },
        Err(err) => Err(anyhow!(err)),
    }
}

fn rusqlite_insert_picture_entry(&self, picture_entry: PictureEntry) -> Result<usize,Error> {
    self.connection.execute(
    "INSERT INTO Picture          \n\
    (File_path,                   \n\
     File_size,                   \n\
     Colors,                      \n\
     Modified_time,               \n\
     Rank,                        \n\
     Label,                       \n\
     Selected,                    \n\
     Deleted,                     \n\
     Cover,                       \n\
     Palette)                     \n\
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10);",
     params![
     picture_entry.file_path,
     picture_entry.file_size as i64,
     picture_entry.image_data.colors as i64,
     picture_entry.modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
     picture_entry.image_data.rank as i64,
     picture_entry.label(),
     picture_entry.image_data.selected as i64,
     picture_entry.deleted as i64,
     picture_entry.image_data.cover as i64,
     palette_to_blob(&picture_entry.image_data.palette)])
}

pub fn insert_new_picture_entry(&self, picture_entry: PictureEntry) -> Result<()> {
    match self.rusqlite_insert_picture_entry(picture_entry) {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(err)),
    }
}

pub fn insert_picture_entry(&self, file_path: &str) -> Result<PictureEntry> {
    match PictureEntry::from_file(file_path) {
        Ok(picture_entry) => {
            match self.rusqlite_insert_picture_entry(picture_entry.clone()) {
                Ok(_) => Ok(picture_entry),
                Err(err) => Err(anyhow!(err)),
            }
        },
        Err(err) => Err(anyhow!(err)),
    }
}

pub fn insert_new_picture_with_file_path(&self, picture_entry: &PictureEntry, file_path: &str) -> Result<()> {
    let new_entry = make_picture_entry(
        file_path.to_string(),
        picture_entry.file_size,
        picture_entry.modified_time,
        ImageData {
            colors: picture_entry.image_data.colors,
            rank: picture_entry.image_data.rank,
            selected: false,
            palette: picture_entry.image_data.palette,
            label: picture_entry.label().unwrap_or_default(),
            cover: false,
            tags: picture_entry.image_data.tags.clone(),
        },
        false);
    self.insert_new_picture_entry(new_entry)
}


pub fn retrieve_or_insert_picture_entry(&self, file_path: &str) -> Result<Option<PictureEntry>> {
    match self.connection.prepare(" SELECT File_Path, File_Size, Colors, Modified_Time, Rank, Palette, Label, Selected, Deleted, Cover, rowid FROM Picture WHERE File_Path = ?1;") {
        Ok(mut statement) => match statement.query([file_path]) {
            Ok(mut rows) => match rows.next() {
                Ok(Some(row)) => match Self::sql_to_picture_entry(row) {
                    Ok(mut entry) => match self.entry_tags(&entry.file_path) {
                        Ok(labels) => {
                            entry.image_data.tags = labels;
                            Ok(Some(entry))
                        },
                        Err(err) => Err(anyhow!(err)),
                    },
                    Err(err) => Err(anyhow!(err)),
                },
                Ok(None) => {
                    if !standard_directory().is_empty() && file_path_directory(file_path) == standard_directory() {
                        match self.insert_picture_entry(file_path) {
                            Ok(picture_entry) => Ok(Some(picture_entry)),
                            Err(err) => Err(anyhow!(err)),
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

