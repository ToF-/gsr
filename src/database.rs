use std::collections::HashSet;
use std::hash::RandomState;
use std::collections::hash_set::Difference;
use crate::picture_io::read_file_info;
use crate::picture_io::get_palette_from_picture;
use crate::image_data::ImageData;
use crate::picture_entry::make_picture_entry;
use std::io;
use rusqlite::DatabaseName;
use rusqlite::types::Type::Blob;
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

    pub fn initialize() -> Result<Self> {
        if let Ok(connection_string) = &env::var(DATABASE_CONNECTION) {
            let path = Path::new(connection_string);
            if path.exists() {
                match Connection::open(connection_string) {
                    Ok(connection) => {
                        println!("opening {}", connection_string);
                        return Ok(Database {
                            connection_string: connection_string.to_string(),
                            connection: connection,
                        })
                    },
                    Err(err) => return Err(anyhow!(err)),
                }
            } else {
                Err(anyhow!("the database file {} can't be opened", connection_string))
            }
        } else {
            Err(anyhow!("the database connection string can't be read. Did you define GALLSHDB?"))
        }
    }

    // database population policy: if the list of pictures currently in the catalog differs from the
    // list of pictures in the database, one should be asked if any update is in order
    // on confirmation, adding those pictures that are in the catalog, into the database
    // on confirmation, removing those pictures that are in the database and not in the catalog
    // these actions should also be available on command line argument

    pub fn update_database(&self, catalog: &Catalog) -> Result<()> {
        let catalog_set: HashSet<String> = HashSet::from_iter(catalog.entries().iter().map(|e| e.file_path.clone()));
        let mut database_set: HashSet<String> = HashSet::new();
        let query = "SELECT file_path from Picture";
        match self.connection.prepare(query) {
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
        let catalog_difference = catalog_set.difference(&database_set);
        if catalog_difference.clone().count() > 0 {
            println!("pictures in this selection that are not in the database:");
            for x in catalog_difference.clone() {
                println!("{x}");
            }
            println!("insert image data for these pictures in the database ?");
            let mut response = String::new();
            let stdin = io::stdin();
            stdin.read_line(&mut response).expect("can't read from stdin");
            match response.chars().next() {
                Some(ch) if ch == 'y' || ch == 'Y' => {
                    match self.populate(catalog, Some(HashSet::from_iter(catalog_difference.into_iter()))) {
                        Ok(()) => {},
                        Err(err) => return Err(anyhow!(err)),
                    }
                },
                Some(_)| None => {},
            }
        }
        println!("pictures in the database that are not in this selection:");
        for x in database_set.difference(&catalog_set) {
            println!("{x}");
        }
        Ok(())
    }

        pub fn check_create_schema(&self, catalog: &Catalog) -> Result<()> {
        println!("checking database {} picture table", self.connection_string);
        let query = "SELECT file_path from Picture";
        let mut count: i64 = 0;
        match self.connection.prepare(query) {
            Ok(_) => self.update_database(catalog),
            Err(err) => Err(anyhow!(err)),
        }
    }

    fn populate(&self, catalog: &Catalog, difference_opt: Option<HashSet<&String>>) -> Result<()> {
        let mut count: usize = 0;
        let total = match difference_opt {
            Some(ref difference) => difference.len(),
            None => catalog.length(),
        };
        for entry in catalog.entries() {
            let insertion:bool = match difference_opt {
                Some(ref set) => set.contains(&entry.file_path),
                None => false,
            };
            if insertion {
                println!("inserting {}", entry.file_path);
                match self.connection.execute("INSERT INTO Picture VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
                params![&*entry.file_path,
                entry.file_size as i64,
                entry.colors as i64,
                entry.modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
                entry.rank as i64,
                palette_to_blob(&entry.palette),
                &*entry.label,
                entry.selected as i64,
                entry.selected as i64]) {
                    Ok(size) => {
                        println!("{}", size);
                    },
                    Err(err) => return Err(anyhow!(err)),
                };
                println!("{}/{}", count, total);
                count += 1;
            }
        };
        Ok(())
    }

    pub fn update_image_data(&self, entry: &PictureEntry) -> Result<()> {
        match self.connection.execute(" UPDATE Picture SET File_Size = ?1, Colors = ?2, Modified_Time = ?3, Rank = ?4, Palette = ?5, Label = ?6, Selected = ?7, Deleted = ?8 WHERE File_Path = ?9;",
        params![entry.file_size as i64,
        entry.colors as i64,
        entry.modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
        entry.rank as i64,
        palette_to_blob(&entry.palette),
        &*entry.label,
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
                        let entry = make_picture_entry(file_path.to_string(), file_size, image_data.colors, modified_time, image_data.rank, Some(image_data.palette), Some(image_data.label), image_data.selected);
                        match self.connection.execute(" INSERT INTO Picture 
            (File_path, File_size, Colors, Modified_time, Rank, Label, Selected, Deleted, Palette)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
            params![
            entry.file_path,
            entry.file_size as i64,
            entry.colors as i64,
            entry.modified_time.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            entry.rank as i64,
            entry.label,
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

    pub fn retrieve_or_create_image_data(&self, file_path: &str) -> Result<PictureEntry> {
        match self.connection.prepare(" SELECT File_Size, Colors, Modified_Time, Rank, Palette, Label, Selected, Deleted, rowid FROM Picture WHERE File_Path = ?1;") {
            Ok(mut statement) => {
                let mut rows = statement.query([file_path])?;
                match rows.next() {
                    Ok(Some(row)) => {
                        Ok(PictureEntry {
                            file_path: file_path.to_string(),
                            file_size: row.get(0)?,
                            colors: row.get(1)?,
                            modified_time: UNIX_EPOCH + Duration::new(row.get::<usize, i64>(2)? as u64, 0),
                            rank: {
                                let result:i64 = row.get(3)?;
                                Rank::from(result)
                            },
                            palette: {
                                let blob : Vec<u8> = row.get(4)?;
                                let mut bytes: [u8;36] = [0;36];
                                for i in 0..36 {
                                    bytes[i] = blob[i];
                                }
                                blob_to_palette(&bytes)
                            },
                            label: row.get(5)?,
                            selected: {
                                let result:bool = row.get(6)?;
                                result
                            },
                            deleted: {
                                let result:bool = row.get(7)?;
                                result
                            },
                        })
                    },
                    Ok(None) => {
                        println!("couldn't find {} in database; insert image data from this file ?", file_path);
                        let mut response = String::new();
                        let stdin = io::stdin();
                        stdin.read_line(&mut response).expect("can't read from stdin");
                        match response.chars().next() {
                            Some(ch) if ch == 'y' || ch == 'Y' => {
                                self.insert_image_data(file_path)
                            },
                            Some(_)|
                            None => Err(anyhow!(format!("couldn't find {} in database", file_path))),
                        }
                    },
                    Err(err) => Err(anyhow!(err)),
                }
            },
            Err(err) => Err(anyhow!(err)),
        }
    }

}

