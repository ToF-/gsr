use crate::palette::palette_to_blob;
use rusqlite::{params, Connection};
use std::time::UNIX_EPOCH;
use std::path::Path;
use anyhow::{anyhow, Result};
use std::env;
use crate::Catalog;

const DATABASE_CONNECTION: &str = "GALLSHDB";

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
                let picture_iter = statement.query_map([], |row| {
                    println!("file_path={}", row.get::<usize, String>(0).unwrap());
                    count +=1;
                    Ok(())
                })?;
                eprintln!("{} records in the picture table. Populating the table.", count);
                self.populate_tables(catalog)
            },
            Err(err) => {
                Err(anyhow!(err))
            }
        }
    }

    fn populate_tables(&self, catalog: &Catalog) -> Result<()> {
        let mut count: usize = 0;
        let total = catalog.entries().len();
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

}

