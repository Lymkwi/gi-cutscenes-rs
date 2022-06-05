//! Module containing various validators for command line arguments

use std::{
    path::PathBuf,
    io::{
        Error,
        ErrorKind,
        Result
    }
};

/// Verify that the provided path exists
fn exists<T: Into<PathBuf>>(path: T) -> Result<PathBuf>
    where PathBuf: From<T>
{
    // 1. Can you build a path ?
    let path = PathBuf::from(path);
    // 2. Does the path exist ?
    if !path.exists() {
        return Err(Error::new(ErrorKind::NotFound, "File not found"));
    }
    Ok(path)
}

/// Verify that the provided path points to an existing file
pub fn is_file<T: Into<PathBuf>>(path: T) -> Result<PathBuf>
    where PathBuf: From<T>
{
    let path = exists(path)?;
    // 3. Is it a file ?
    if !path.is_file() {
        // When https://github.com/rust-lang/rust/issues/86442 closes,
        // replace this with ErrorKind::IsADirectory
        return Err(Error::new(ErrorKind::Unsupported, "File is a directory"));
    }
    Ok(path)
}

/// Verify that the provided path has the corresponding extension
pub fn has_right_extension<T: Into<PathBuf>>(path: T, ext: &str) -> Result<()>
    where PathBuf: From<T>
{
    let path = is_file(path)?;
    // 4. Does it end in USM ?
    if path.extension() != Some(std::ffi::OsStr::new(ext)) {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid file extension"));
    }

    // No need to check read access, we'll know if we can't read later when we try
    Ok(())
}

pub fn is_usm_file<T: Into<PathBuf>>(path: T) -> Result<()>
    where PathBuf: From<T>
{
    has_right_extension(path, "usm")
}

pub fn is_hca_file<T: Into<PathBuf>>(path: T) -> Result<()>
    where PathBuf: From<T>
{
    has_right_extension(path, "hca")
}

/// Verify that the path provided points to an existing directory
pub fn is_dir<T: Into<PathBuf>>(path: T) -> Result<()>
    where PathBuf: From<T>
{
    let path = exists(path)?;
    // 3. Is it a file ?
    if !path.is_dir() {
        // When https://github.com/rust-lang/rust/issues/86442 closes,
        // replace this with ErrorKind::NotADirectory
        return Err(Error::new(ErrorKind::Unsupported, "Not a directory"));
    }
    // Again, read/write access is not checked, we will find out when we try
    Ok(())
}