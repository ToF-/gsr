use std::path::Path;
use sqlite::State;
use anyhow::{anyhow, Result};
use sqlite::Connection;
use sqlite::Value;
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
                match sqlite::open(connection_string) {
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
                while let Ok(State::Row) = statement.next() {
                    println!("file_path={}", statement.read::<String, _>("file_path").unwrap());
                    count +=1;
                }
                eprintln!("{} records in the picture table", count);
                Ok(())
            },
            Err(err) => {
                Err(anyhow!(err))
            }
        }
    }

    fn populate_tables(&self, catalog: &Catalog) -> Result<()> {
        for entry in catalog.entries() {
            let query = "INSERT INTO Picture VALUES (:File_Path, :File_Size, :Colors, :Modified_Time, :Rank, :Selected, :Palette_0, :Palette_1, :Palette_2, :Palette_3, :Palette_4, :Palette_5, :Palette_6, :Palette_7, :Palette_8, :Label, :Selected, :Deleted);";
            let mut statement = self.connection.prepare(query)?;
            statement.bind_iter::<_, (_, Value)>([
                (":File_Path", entry.file_path.into()),
                (":File_Size", entry.file_size),
                (":Colors", entry.colors.try_into().unwrap()),
                (":Modified_Time", entry.modified_time.into()),
                (":Rank", entry.rank.into()),
                (":Palette_0", entry.palette[0].into()),
                (":Palette_1", entry.palette[1].into()),
                (":Palette_2", entry.palette[2].into()),
                (":Palette_3", entry.palette[3].into()),
                (":Palette_4", entry.palette[4].into()),
                (":Palette_5", entry.palette[5].into()),
                (":Palette_6", entry.palette[6].into()),
                (":Palette_7", entry.palette[7].into()),
                (":Palette_8", entry.palette[8].into()),
                (":Selected", entry.selected.into()),
                (":Deleted", entry.deleted.into())
            ])?;
        };
        Ok(())
    }

}

