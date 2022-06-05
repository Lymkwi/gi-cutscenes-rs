use std::{
    io::{
        Error,
        ErrorKind,
        Result
    },
    path::Path
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Data {
    version: String,
    videos: Vec<String>,
    key: u64
}

pub fn read_version_file<T: AsRef<Path>>(path: T) -> Result<Vec<Data>>
{
    let content = std::fs::read(path)?;
    let data: String = String::from_utf8(content)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    let parsed: std::collections::HashMap<String, Vec<Data>> = serde_json::from_str(&data)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    for (key, val) in parsed {
        if key == "list" {
            return Ok(val);
        }
    }
    Err(Error::new(ErrorKind::InvalidData, "Missing list key"))
}