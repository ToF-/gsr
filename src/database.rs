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

    pub fn check_create_schema(&self, catalog: &Catalog) -> Result<()> {
        println!("checking database {} picture table", self.connection_string);
        let query = "SELECT file_path from Picture";
        let mut count: i64 = 0;
        match self.connection.prepare(query) {
            Ok(mut statement) => {
                let file_paths = statement.query_map([], |row| {
                    let file_path = row.get::<usize, String>(0).unwrap();
                    Ok(file_path)
                })?;
                for file_path in file_paths {
                    println!("{:?}", file_path);
                    count += 1;
                };
                if count == 0 {
                    eprintln!("{} records in the picture table. Populating the table.", count);
                    self.populate(catalog)
                } else {
                    eprintln!("{} records in the picture table.", count);
                    Ok(())
                }
            },
            Err(err) => {
                Err(anyhow!(err))
            },
        }
    }

    fn populate(&self, catalog: &Catalog) -> Result<()> {
        let mut count: usize = 0;
        let total = catalog.entries().len();
        println!("{} records to insert", total);
        for entry in catalog.entries() {
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

    pub fn retrieve_or_create_image_data(&self, file_path: &str) -> Result<PictureEntry> {
        let mut statement = self.connection.prepare(" SELECT File_Size, Colors, Modified_Time, Rank, Palette, Label, Selected, Deleted, rowid FROM Picture WHERE File_Path = ?1;")?;
        let mut rows = statement.query([file_path])?;
        if let Ok(Some(row)) = rows.next() {

            let rowid: i64 = row.get(8)?;
            let blob = self.connection.blob_open(DatabaseName::Main, "Picture", "palette", rowid, true)?;
            let mut blob_data: [u8;36] = [0; 36];
            blob.read_at(&mut blob_data, 0)?;
            Ok(PictureEntry {
                file_path: file_path.to_string(),
                file_size: row.get(0)?,
                colors: row.get(1)?,
                modified_time: UNIX_EPOCH + Duration::new(row.get::<usize, i64>(2)? as u64, 0),
                rank: {
                    let result:i64 = row.get(3)?;
                    Rank::from(result)
                },
                palette: blob_to_palette(&blob_data),
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
        } else {
            Err(anyhow!("couldn't find {} in database", file_path))
        }
    }

}

