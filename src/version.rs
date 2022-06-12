use std::{
    io::{
        Error,
        ErrorKind,
        Result
    },
    path::Path
};
use serde::Deserialize;

use crate::errors::{
    GICSError,
    GICSResult
};

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
pub struct Data {
    version: String,
    videos: Vec<String>,
    key: u64
}

impl Data {
    pub const fn get_key(&self) -> u64 {
        self.key
    }

    pub fn contains_video(&self, name: &str) -> bool {
        let name_str = name.into();
        self.videos.contains(&name_str)
    }
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

pub fn definite_version_keys(filename: &str, version_keys: Option<&[Data]>, key1: Option<u32>, key2: Option<u32>) -> GICSResult<(u32, u32)> {
    match (key1, key2) {
        (Some(k1), Some(k2)) => Ok((k2, k1)),
        _ => version_keys.map_or_else(
            || Err(GICSError::new("No key file provided and at least one of the keys missing : unable to extract keys.")),
            |vkeys| find_version_keys(vkeys, filename)
        ) 
    }
}

fn find_version_keys(version_keys: &[Data], file: &str) -> GICSResult<(u32, u32)> {
    find_key(file, version_keys).map(split_key)
}

fn find_key(filename: &str, version_keys: &[Data]) -> GICSResult<u64>
{
    let key1: u64 = file_name_encryption_key(filename);
    //let (key2, bld) = blk_encryption_key(filename);
    let key2: u64 = blk_encryption_key(filename, version_keys)?;

    //if (key1 + key2 & 0xFF_FFFF_FFFF_FFFF) == 0 {
    Ok(if (key1+key2).trailing_zeros() >= 56 {
        0x100_0000_0000_0000
    } else {
        (key1 + key2) & 0xFF_FFFF_FFFF_FFFF
    })
}

fn blk_encryption_key(filename: &str, version_keys: &[Data]) -> GICSResult<u64>
{
    let basename: &str = filename.split('.').next().unwrap();
    // Look into the version_keys
    for data in version_keys {
        if data.contains_video(basename) {
            return Ok(data.get_key())
        }
    }
    Err(GICSError::new(&format!("Unable to find decryption key for \"{}\" in version keys file", basename)))
}

// The intro cutscenes are not exactly encrypted, because the game needs to be able to play
// Them without having downloaded all of the streaming assets
const INTROS: [&str; 3] = [ "MDAQ001_OPNew_Part1.usm", "MDAQ001_OPNew_Part2_PlayerBoy.usm", "MDAQ001_OPNew_Part2_PlayerGirl.usm" ];

fn file_name_encryption_key(filename: &str) -> u64
{
    let filename: &str = if INTROS.contains(&filename) { "MDAQ001_OP" } else { filename };
    // Now, get the front of the file without the extension
    let basename: &str = filename.split('.').next().unwrap();
    let mut sum: u64 = 0;

    sum = basename.bytes().fold(sum, |acc, bt| acc.wrapping_mul(3).wrapping_add(u64::from(bt)));

    sum &= 0xFF_FFFF_FFFF_FFFF;
    if sum > 0 { sum } else { 0x100_0000_0000_0000 }
}

fn split_key(key: u64) -> (u32, u32)
{
    ((key >> 32).try_into().unwrap(), (key & 0xffff_ffff).try_into().unwrap())
}