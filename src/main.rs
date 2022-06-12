//! CI Cutscenes Extractor : Rust Remix

// Make clippy quite nasty
#![deny(clippy::cargo)]         // Checks for garbage in the Cargo TOML files
#![deny(clippy::complexity)]    // Checks for needlessly complex structures
#![deny(clippy::correctness)]   // Checks for common invalid usage and workarounds
#![deny(clippy::nursery)]       // Checks for things that are typically forgotten by learners
#![deny(clippy::pedantic)]      // Checks for mildly annoying comments it could make about your code
#![deny(clippy::perf)]          // Checks for inefficient ways to perform common tasks
#![deny(clippy::style)]         // Checks for inefficient styling of code
#![deny(clippy::suspicious)]    // Checks for potentially malicious behaviour
// Add some new clippy lints
#![deny(clippy::use_self)]      // Checks for the use of a struct's name in its `impl`
// Add some default lints
#![deny(unused_variables)]      // Checks for unused variables
// Deny missing documentation
#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]

// Everything allowed here will go because it's bad number type management
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

//#![allow(dead_code)]

use clap::{
    Arg,
    Command, ValueHint
};

use std::path::{
    Path,
    PathBuf
};

mod demux;
mod filetypes;
mod errors;
mod tools;
mod validate;
mod version;

#[allow(clippy::too_many_lines)]
fn main() {
    let key1 = Arg::new("key1")
        .short('a')
        .long("key1")
        .value_name("key1")
        .help("4 lower bytes of the key")
        .takes_value(true)
        .validator(|s| u32::from_str_radix(s, 16))
        .value_hint(ValueHint::Other);
    let key2 = Arg::new("key2")
        .short('b')
        .long("key2")
        .value_name("key2")
        .help("4 higher bytes of the key")
        .takes_value(true)
        .validator(|s| u32::from_str_radix(s, 16))
        .value_hint(ValueHint::Other);
    let subs_option = Arg::new("subs")
        .short('s')
        .long("subtitles")
        .help("Adds the substitles to the MKV file");
    let merge_option = Arg::new("merge")
        .short('m')
        .long("merge")
        .help("Merges the extracted content into a MKV container file");
    let ffmpeg_option = Arg::new("merge-program")
        .short('p')
        .long("merge-program")
        .takes_value(true)
        .default_value("ffmpeg")
        .help("Path to the ffmpeg binary used to merge the output files");
    let version_json = Arg::new("version-keys")
        .short('k')
        .long("version-keys")
        .help("Provide a path to a valid version.json file containing the version keys")
        .takes_value(true)
        .default_value("version.json");
    let args = Command::new("GI Cutscenes")
        .version("0.0.2")
        .author("Lux A. Phifollen <limefox@vulpinecitrus.info>")
        .about("Command-line utility to extract and demux GI cutscenes")
        // See https://users.rust-lang.org/t/clap-how-to-group-require-top-level-subcommands/24789
        .subcommand_required(true)
        .subcommand(
            Command::new("demuxUsm")
                .about("Demuxes a specified .usm file to a specified folder")
                .arg(Arg::new("demux-file")
                    .short('f')
                    .long("demux-file")
                    .value_name("demux_file")
                    .help("File to read and display on the console")
                    .required(true)
                    .takes_value(true)
                    .value_hint(ValueHint::FilePath)
                    .validator(|s| validate::is_usm_file(s)))
                .arg(key1.clone())
                .arg(key2.clone())
                .arg(version_json.clone())
                .arg(subs_option.clone())
                .arg(merge_option.clone())
                .arg(ffmpeg_option.clone())
        )
        .subcommand(
            Command::new("batchDemux")
                .about("Tries to demux all .usm files in the specified folder")
                .arg(Arg::new("usm-folder")
                    .short('u')
                    .long("usm-folder")
                    .value_name("usm_folder")
                    .help("Folder containing the .usm files to be demuxed")
                    .takes_value(true)
                    .validator(|s| validate::is_dir(s))
                    .value_hint(ValueHint::DirPath))
                .arg(version_json.clone())
                .arg(subs_option)
                .arg(merge_option)
                .arg(ffmpeg_option)
        )
        .subcommand(
            Command::new("convertHca")
                .about("Converts input .hca files into .wav files")
                .arg(Arg::new("hca-input")
                    .short('i')
                    .long("hca-input")
                    .value_name("hca_input")
                    .help("File or directory to be processed")
                    .takes_value(true)
                    .validator(|s| validate::is_hca_file(s))
                    .value_hint(ValueHint::FilePath))
                .arg(Arg::new("basename")
                    .short('n')
                    .long("base-name")
                    .value_name("base_name")
                    .help("Base name of the file (to find its cutscene in versions.json)")
                    .takes_value(true)
                )
                .arg(key1)
                .arg(key2)
                .arg(version_json)
        )
        .arg(Arg::new("output")
             .short('o')
             .long("output")
             .value_name("output")
             .help("Output folder")
             .value_hint(ValueHint::DirPath))
        .arg(Arg::new("no-cleanup")
            .visible_alias("nc")
            .long("no-cleanup")
            .help("Keeps the extracted files instead of removing them"))
        .get_matches();

    let cleanup: bool = !args.is_present("no-cleanup");

    match args.subcommand() {
        Some(("demuxUsm", cmd)) => {
            // Start to extract the arguments
            // Clap already validated the paths and the key values if any
            let file: PathBuf = PathBuf::from(cmd.value_of("demux-file").unwrap());
            let basename: String = file
                .file_name().expect("No file name in the provided path")
                .to_str().expect("Unable to decode path name into UTF-8")
                .into();
            let key_one: Option<u32> = cmd.value_of("key1").map(|s| u32::from_str_radix(s, 16).unwrap());
            let key_two: Option<u32> = cmd.value_of("key2").map(|s| u32::from_str_radix(s, 16).unwrap());
            let subs: bool = cmd.is_present("subs"); // This will become a path later
            let merge: bool = cmd.is_present("merge");
            let ffmpeg_path: &str = cmd.value_of("merge-program").unwrap();
            let output: PathBuf = args.value_of("output")
                // No need to re-validate since we know the file is good, its folder must be too
                .map_or_else(
                    // If we do not get an output file, place the output alongside our input file
                    || {
                        let mut def = file.clone();
                        def.set_extension("mkv");
                        def
                    },
                    // If we do, attempt to get a path from it
                    PathBuf::from
                );

            // We haven't validated the file here
            let version_file: &str = cmd.value_of("version-keys").unwrap();
            let version_keys: Option<Vec<version::Data>> = if key_one.is_none() || key_two.is_none() {
                    // Let's validate that the file exists
                    if let Err(e) = validate::is_file(version_file) {
                        eprintln!("Error opening version keys file : {}", e);
                        return;
                    }
                    match version::read_version_file(PathBuf::from(version_file)) {
                        Ok(keydata) => Some(keydata),
                        Err(e) => {
                            eprintln!("Error reading key file : {}", e);
                            return;
                        }
                    }
                } else { None };

            let (key_two, key_one) = version::definite_version_keys(&basename, version_keys.as_deref(), key_one, key_two).unwrap();
            println!("Keys derived for \"{}\" : ({:08X}, {:08X})", file.to_str().unwrap(), key_two, key_one);
            let res = demux::process_file(file, key_two, key_one, output.as_path(), merge, cleanup, subs, ffmpeg_path);
            if let Err(e) = res {
                eprintln!("Error: {}", e);
            }

        },
        Some(("batchDemux", cmd)) => {
            // Start to extract the arguments
            // Clap already validated the paths and the key values if any
            let folder: PathBuf = PathBuf::from(cmd.value_of("usm-folder").unwrap());
            let subs: bool = cmd.is_present("subs");
            let merge: bool = cmd.is_present("merge");
            let ffmpeg_path: &str = cmd.value_of("merge-program").unwrap();
            let output: PathBuf = args.value_of("output")
                // No need to re-validate since we know the file is good, its folder must be too
                .map_or_else(
                    // If we do not get an output folder (or it's invalid), place the output alongside our input file
                    || folder.clone(),
                    // If we do, attempt to get a path from it
                    PathBuf::from
                );

            // We haven't validated this json, we need to. We just know it's a file that exists
            let version_file: &str = cmd.value_of("version-keys").unwrap();
            let version_keys: Vec<version::Data> = match version::read_version_file(PathBuf::from(version_file)) {
                Ok(keydata) => keydata,
                Err(e) => {
                    eprintln!("Error reading key file : {}", e);
                    return;
                }
            };

            // Start working through the directory..
            if let Err(e) = demux::process_directory(folder, &version_keys, output.as_path(), merge, cleanup, subs, ffmpeg_path) {
                eprintln!("Error: {}", e);
            }
        },
        Some(("convertHca", cmd)) => {
            // Start to extract the arguments
            // Clap already validated the paths and the key values if any
            let file: PathBuf = PathBuf::from(cmd.value_of("hca-input").unwrap());
            let basename: &str = cmd.value_of("basename")
                .unwrap_or_else(||
                    file
                        .file_name().expect("No file name in the provided path")
                        .to_str().expect("Unable to decode path name into UTF-8")
                );
            let output: PathBuf = args.value_of("output")
                // No need to re-validate since we know the file is good, its folder must be too
                .map_or_else(
                    // If we do not get an output file path (or it's invalid), place the output alongside our input file
                    || {
                        let mut def = file.clone();
                        def.set_extension("wav");
                        def
                    },
                    // If we do, attempt to get a path from it
                    PathBuf::from
                );
            let key_one: Option<u32> = cmd.value_of("key1").map(|s| u32::from_str_radix(s, 16).unwrap());
            let key_two: Option<u32> = cmd.value_of("key2").map(|s| u32::from_str_radix(s, 16).unwrap());
            // We haven't validated the file here
            let version_file: &str = cmd.value_of("version-keys").unwrap();
            let version_keys: Option<Vec<version::Data>> = if key_one.is_none() || key_two.is_none() {
                    // Let's validate that the file exists
                    if let Err(e) = validate::is_file(version_file) {
                        eprintln!("Error opening version keys file : {}", e);
                        return;
                    }
                    match version::read_version_file(PathBuf::from(version_file)) {
                        Ok(keydata) => Some(keydata),
                        Err(e) => {
                            eprintln!("Error reading key file : {}", e);
                            return;
                        }
                    }
                } else { None };

            // Get our keys
            let (key_one, key_two) = version::definite_version_keys(basename, version_keys.as_deref(), key_one, key_two).unwrap();

            // Convert
            if let Err(e) = demux::process_hca(file, key_two, key_one, output.as_path(), cleanup) {
                eprintln!("Error: {}", e);
            }
        },
        _ => { eprintln!("No subcommand provided"); }
    }
}
