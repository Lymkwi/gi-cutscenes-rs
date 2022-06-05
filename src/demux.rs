use std::{
    io::{Error, ErrorKind},
    path::{
        Path,
        PathBuf
    }
};

use crate::filetypes::{
    HCAFile,
    USMFile
};

pub trait Demuxable {
    fn demux(self, video_extract: bool, audio_extract: bool, output: &Path) -> std::io::Result<(PathBuf, Vec<PathBuf>)>;
}

use crate::version;

// The intro cutscenes are not exactly encrypted, because the game needs to be able to play
// Them without having downloaded all of the streaming assets
const INTROS: [&str; 3] = [ "MDAQ001_OPNew_Part1.usm", "MDAQ001_OPNew_Part2_PlayerBoy.usm", "MDAQ001_OPNew_Part2_PlayerGirl.usm" ];

fn split_key(key: u64) -> (u32, u32)
{
    ((key >> 32).try_into().unwrap(), (key & 0xffff_ffff).try_into().unwrap())
}

fn file_name_encryption_key(filename: &str) -> u64
{
    let filename: &str = if INTROS.contains(&filename) { "MDAQ001_OP" } else { filename };
    // Now, get the front of the file without the extension
    let basename: &str = filename.split('.').next().unwrap();
    let mut sum: u64 = 0;

    sum = basename.bytes().fold(sum, |acc, bt| 3 * acc + u64::from(bt));

    sum &= 0xFF_FFFF_FFFF_FFFF;
    if sum > 0 { sum } else { 0x100_0000_0000_0000 }
}

fn blk_encryption_key(filename: &str, _version_keys: &[version::Data]) -> u64
{
    let _basename: &str = filename.split('.').next().unwrap();
    0
}

fn find_key(filename: &str, version_keys: &[version::Data]) -> u64
{
    let key1: u64 = file_name_encryption_key(filename);
    //let (key2, bld) = blk_encryption_key(filename);
    let key2: u64 = blk_encryption_key(filename, version_keys);

    //if (key1 + key2 & 0xFF_FFFF_FFFF_FFFF) == 0 {
    if (key1+key2).trailing_zeros() >= 56 {
        0x100_0000_0000_0000
    } else {
        (key1 + key2) & 0xFF_FFFF_FFFF_FFFF
    }
}

pub fn process_file(file: PathBuf, version_keys: Option<Vec<version::Data>>, key2: Option<u32>, key1: Option<u32>, output: &Path) -> Result<(), Error> {
    // Step 1 : What is the file name ?
    let filename: String = file.file_name().unwrap().to_str().unwrap().into();
    // Step 2 : Do we have keys ?
    let (key2, key1) = match (key2, key1) {
        (Some(k2), Some(k1)) => (k2, k1),
        _ => {
            match version_keys {
                Some(v) => split_key(find_key(&filename, &v)),
                None => {
                    return Err(Error::new(ErrorKind::NotFound, "No keys provided, and no version keyfile provided"));
                }
            }
            
        }
    };
    println!("Keys derived for \"{}\" : ({:08X}, {:08X})", file.to_str().unwrap(), key2, key1);
    let file: USMFile = USMFile::new(file, key2.to_le_bytes(), key1.to_le_bytes());
    let (_video_path, audio_path_vec) = file.demux(true, true, output)?;
    for audio_path in audio_path_vec {
        let audio_file: HCAFile = HCAFile::new(audio_path.clone(), key2.to_le_bytes(), key1.to_le_bytes())?;
        audio_file.convert_to_wav(&audio_path)?;
    }
    Ok(())
}