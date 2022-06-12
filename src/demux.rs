use std::path::{
    Path,
    PathBuf
};

use crate::{
    errors::{
        GICSError,
        GICSResult
    },
    filetypes::{
        HCAFile,
        USMFile,
        MKVFile
    },
    version::{
        definite_version_keys,
        Data
    }
};

pub trait Demuxable {
    fn demux(self, video_extract: bool, audio_extract: bool, output: &Path) -> GICSResult<(PathBuf, Vec<PathBuf>)>;
}

fn output_paths_from(file: &Path, merge: bool, output: &Path) -> GICSResult<(PathBuf, PathBuf)> {
    let dot = PathBuf::from(".");
    let base_name = file
        .file_name().ok_or_else(|| GICSError::new("Unable to retrieve input file base name"))?
        .to_str().ok_or_else(|| GICSError::new("Unable to decode base name to UTF-8"))?;
    let mut tentative_mkv = PathBuf::from(output);
    let mut tentative_out = PathBuf::from(output);
    if merge {
        // The output has to have "MKV" as its extension, or be a directory
        if output.extension().is_none() {
            // Does it exist? Is it a directory?
            if output.exists() {
                if !output.is_dir() {
                    return Err(GICSError::new("Output path is not a directory and exists"));
                }
            } else {
                std::fs::create_dir_all(output)?;
            }
            tentative_mkv.push(base_name);
            tentative_mkv.set_extension("mkv");
        } else if output.extension().unwrap()
            .to_str().ok_or_else(|| GICSError::new("File extension is not UTF-8"))?
            != "mkv" {
                return Err(GICSError::new("Merge option provided but output file name isn't a mkv file"))
        } else {
            // it has an MKV extension
            tentative_out = PathBuf::from(tentative_out.parent().unwrap_or(dot.as_path()));
        }
    } else {
        // It has to be a directory, potentially create the output directories
        if output.exists() {
            if !output.is_dir() {
                return Err(GICSError::new("Unable to treat the output as a directory"));
            }
        } else {
            std::fs::create_dir_all(output)?;
        }
    }
    Ok((tentative_out, tentative_mkv))
}

#[allow(clippy::too_many_arguments)]
pub fn process_file(file: PathBuf, key2: u32, key1: u32, output: &Path, merge: bool, cleanup: bool, _subs: bool, ffmpeg_path: &str) -> GICSResult<()> {
    let (output_directory, mkv_output) = output_paths_from(file.as_path(), merge, output)?;

    let file: USMFile = USMFile::new(file, key2.to_le_bytes(), key1.to_le_bytes());
    let (video_path, audio_path_vec) = file.demux(true, true, output_directory.as_path())?;
    println!("File demuxed. Collected one video and {} audio files.", audio_path_vec.len());

    // Convert the HCAs right now
    let a_paths = audio_path_vec.into_iter().enumerate().map(|(id, audio_path)| {
            let basename: String = audio_path
                .file_name().ok_or_else(|| GICSError::new("No file name in the provided path"))?
                .to_str().ok_or_else(|| GICSError::new("Unable to decode path name into UTF-8"))?
                .into();
            let mut audio_output = output_directory.clone();
            audio_output.push(&basename);
            audio_output.set_extension("wav");
            println!("Processing track #{}..", id);
            process_hca(audio_path.clone(), key2, key1, audio_output.as_path(), cleanup)
        }).collect::<GICSResult<Vec<PathBuf>>>()?;

    if merge {
        MKVFile::attempt_merge(
            mkv_output,
            video_path.as_path(),
            &a_paths.iter().map(PathBuf::as_path).collect::<Vec<&Path>>(),
            ffmpeg_path
        )?;
        if cleanup {
            // Remove video
            std::fs::remove_file(video_path)?;
            a_paths.iter()
                .map(|p| std::fs::remove_file(p).map_err(GICSError::from))
                .collect::<GICSResult<Vec<()>>>()?;
        }
        Ok(())
    } else {
        Ok(())
    }
}

pub fn process_hca(file: PathBuf, key2: u32, key1: u32, output: &Path, cleanup: bool) -> GICSResult<PathBuf> {
    let audio_file: HCAFile = HCAFile::new(file.clone(), key2.to_le_bytes(), key1.to_le_bytes())?;
    let outfile = audio_file.convert_to_wav(output)?;
    if cleanup {
        std::fs::remove_file(file)?;
    }
    Ok(outfile)
}

pub fn process_directory(folder: PathBuf, version_keys: &[Data], output: &Path, merge: bool, cleanup: bool, subs: bool, ffmpeg_path: &str) -> GICSResult<()> {
    if output.exists() && !output.is_dir() {
        println!("{:?}", output);
        return Err(GICSError::new("Provided output path is not a directory; this would overwrite every result. Aborting"));
    }
    for entry in std::fs::read_dir(folder)? {
        let path = entry?.path();
        if !path.is_file() {
            println!("Skipping file \"{}\"", path.to_str().ok_or_else(|| GICSError::new("Unable to decode file path to UTF-8"))?);
            continue;
        }
        // File has to have an extension and be a USM
        if path.extension().is_none()
            || path.extension().unwrap().to_str().is_none()
            || path.extension().unwrap().to_str().unwrap() != "usm" {
                println!("Skipping file \"{}\"", path.to_str().ok_or_else(|| GICSError::new("Unable to decode file path to UTF-8"))?);
                continue;
        }
        // Copy the entry name
        let mut outpath = PathBuf::from(output);
        let basename = path
            .file_name().ok_or_else(|| GICSError::new("Unable to find base name of an entry"))?
            .to_str().ok_or_else(|| GICSError::new("Unable to decode file name to UTF-8"))?;
        outpath.push(basename);
        outpath.set_extension("mkv");
        // Find keys
        let (key_two, key_one) = definite_version_keys(basename, Some(version_keys), None, None)?;
        println!("Keys derived for \"{}\" : ({:08X}, {:08X})", basename, key_two, key_one);
        process_file(path, key_two, key_one, outpath.as_path(), merge, cleanup, subs, ffmpeg_path)?;
    }
    Ok(())
}