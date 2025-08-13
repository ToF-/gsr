use sqlite::State;
use anyhow::{anyhow, Result};
use sqlite::Connection;
use sqlite::Value;
use std::env;
use std::io;
use crate::Catalog;

const DATABASE_CONNECTION: &str = "GALLSHDB";

pub struct Database {
    pub connection_string: String,
    connection: Connection,
}

impl Database {

    pub fn initialize() -> Result<Self> {
        if let Ok(connection_string) = &env::var(DATABASE_CONNECTION) {
            match sqlite::open(connection_string) {
                Ok(connection) => {
                    println!("opening {}", connection_string);
                    Ok(Database {
                        connection_string: connection_string.to_string(),
                        connection: connection,
                    })
                },
                Err(err) => Err(anyhow!(err)),
                }
        } else {
            Err(anyhow!("the database connection string can't be read. Did you define GALLSHDB?"))
        }

    }

    pub fn check_create_schema(&self, catalog: &Catalog) -> Result<()> {
        println!("checking database {} picture table", self.connection_string);
        let query = "SELECT Count(*) from Picture";
        let mut count: i64 = 0;
        let query = "SELECT count(*) FROM Picture";
        match self.connection.prepare(query) {
            Ok(mut statement) => {
                while let Ok(State::Row) = statement.next() {
                    count = statement.read::<i64, _>("count(*)").unwrap();
                }
            },
            Err(err) => {
                println!("{err}");
            }
        };
        if count == 0 {
            println!("the database {} does not contain a Picture table. Create the database tables?", self.connection_string);
            let mut response = String::new();
            let stdin = io::stdin();
            stdin.read_line(&mut response).expect("can't read from stdin");
            match response.chars().next() {
                Some(ch) if ch == 'y' || ch == 'Y' => {
                    println!("creating tables in database {}", self.connection_string);
                    match self.create_tables() {
                        Ok(()) => {
                            return self.populate_tables(catalog)
                        },
                        err => err,
                    }

                },
                _ => Err(anyhow!("database does not contain Picture table")),
            }

        } else {
            Ok(())
        }
    }

    fn create_tables(&self) -> Result<()> {
        let query = "CREATE TABLE Picture ( file_path TEXT PRIMARY KEY );";
        match self.connection.execute(query) {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    fn populate_tables(&self, catalog: &Catalog) -> Result<()> {
        for entry in catalog.entries() {
            let query = "INSERT INTO Picture VALUES (:file_path)";
            let mut statement = self.connection.prepare(query)?;
            statement.bind_iter::<_, (_, Value)>([
                (":file_path", entry.file_path.clone().clone().into()),
            ])?;
        };
        Ok(())
    }

}

