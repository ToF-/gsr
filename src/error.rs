use std::error::Error;
use anyhow::{anyhow, Result};

pub fn error<T: std::fmt::Debug, E: std::error::Error + Error >(e: Result<T>) -> Result<T,anyhow::Error> {
    Err(anyhow!(e.unwrap_err()))
}


