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
const INTROS: [&'static str; 3] = [ "MDAQ001_OPNew_Part1.usm", "MDAQ001_OPNew_Part2_PlayerBoy.usm", "MDAQ001_OPNew_Part2_PlayerGirl.usm" ];

fn split_key(key: u64) -> (u32, u32)
{
    ((key >> 32).try_into().unwrap(), (key & 0xffffffff).try_into().unwrap())
}

fn file_name_encryption_key(filename: &str) -> u64
{
    let filename: &str = if INTROS.contains(&filename) { "MDAQ001_OP" } else { filename };
    // Now, get the front of the file without the extension
    let basename: &str = filename.split('.').next().unwrap();
    let mut sum: u64 = 0;

    sum = basename.bytes().fold(sum, |acc, bt| 3 * acc + (bt as u64));

    sum &= 0xFFFFFFFFFFFFFF;
    if sum > 0 { sum } else { 0x100000000000000 }
}

fn blk_encryption_key(filename: &str, _version_keys: Vec<version::Data>) -> u64
{
    let _basename: &str = filename.split('.').next().unwrap();
    0
}

fn find_key(filename: &str, version_keys: Vec<version::Data>) -> u64
{
    let key1: u64 = file_name_encryption_key(filename);
    //let (key2, bld) = blk_encryption_key(filename);
    let key2: u64 = blk_encryption_key(filename, version_keys);

    if (key1 + key2 & 0xFFFFFFFFFFFFFF) != 0 {
        (key1 + key2) & 0xFFFFFFFFFFFFFF
    } else {
        0x100000000000000
    }
}


fn key_array(key: u32) -> [u8; 4] {
    [
        ((key >> 24) & 0xFF).try_into().unwrap(),
        ((key >> 16) & 0xFF).try_into().unwrap(),
        ((key >>  8) & 0xFF).try_into().unwrap(),
        (key & 0xFF).try_into().unwrap()
    ]
}

pub fn process_file(file: PathBuf, version_keys: Option<Vec<version::Data>>, key2: Option<u32>, key1: Option<u32>, output: PathBuf) -> Result<(), Error> {
    // Step 1 : What is the file name ?
    let filename: String = file.file_name().unwrap().to_str().unwrap().into();
    // Step 2 : Do we have keys ?
    let (key2, key1) = match (key2, key1) {
        (Some(k2), Some(k1)) => (k2, k1),
        _ => {
            match version_keys {
                Some(v) => split_key(find_key(&filename, v)),
                None => {
                    return Err(Error::new(ErrorKind::NotFound, "No keys provided, and no version keyfile provided"));
                }
            }
            
        }
    };
    println!("Keys derived for \"{}\" : ({:X}, {:X})", file.to_str().unwrap(), key1, key2);
    let file: USMFile = USMFile::new(file, key2.to_le_bytes(), key1.to_le_bytes());
    let (_video_path, audio_path_vec) = file.demux(true, true, output.as_path())?;
    for audio_path in audio_path_vec {
        let audio_file: HCAFile = HCAFile::new(audio_path.clone(), key_array(key2), key_array(key1))?;
        audio_file.convert_to_wav(&audio_path)?;
    }
    Ok(())
}