use anyhow::{anyhow,Result};
use std::io;

pub fn prompt_yes_no(message: &str) -> Result<Option<char>> {
    println!("{}", message);
    let mut response = String::new();
    let stdin = io::stdin();
    match stdin.read_line(&mut response) {
        Ok(_) => Ok(response.chars().next()),
        Err(err) => Err(anyhow!(err)),
    }
}


