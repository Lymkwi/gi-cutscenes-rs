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
    let subs_option = Arg::new("subs")
        .short('s')
        .long("subtitles")
        .help("Adds the substitles to the MKV file");
    let merge_option = Arg::new("merge")
        .short('m')
        .long("merge")
        .help("Merges the extracted content into a MKV container file");
    let version_json = Arg::new("version-keys")
        .short('k')
        .long("version-keys")
        .help("Provide a path to a valid version.json file containing the version keys")
        .takes_value(true)
        .default_value("version.json");
    let args = Command::new("GI Cutscenes")
        .version("0.0.1")
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
                .arg(Arg::new("key1")
                    .short('a')
                    .long("key1")
                    .value_name("key1")
                    .help("4 lower bytes of the key")
                    .takes_value(true)
                    .validator(|s| u32::from_str_radix(s, 16))
                    .value_hint(ValueHint::Other))
                .arg(Arg::new("key2")
                    .short('b')
                    .long("key2")
                    .value_name("key2")
                    .help("4 higher bytes of the key")
                    .takes_value(true)
                    .validator(|s| u32::from_str_radix(s, 16))
                    .value_hint(ValueHint::Other))
                .arg(version_json.clone())
                .arg(subs_option.clone())
                .arg(merge_option.clone())
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
                .arg(version_json
                    // Only validate here because we actually need the file
                    .validator(|s| validate::is_file(s)))
                .arg(subs_option)
                .arg(merge_option)
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

    println!("{:?}", args.subcommand());
    let cleanup: bool = !args.is_present("no-cleanup");

    match args.subcommand() {
        Some(("demuxUsm", cmd)) => {
            // Start to extract the arguments
            // Clap already validated the paths and the key values if any
            let file: PathBuf = PathBuf::from(cmd.value_of("demux-file").unwrap());
            let key_1: Option<u32> = cmd.value_of("key1").map(|s| u32::from_str_radix(s, 16).unwrap());
            let key_2: Option<u32> = cmd.value_of("key2").map(|s| u32::from_str_radix(s, 16).unwrap());
            let subs: bool = cmd.is_present("subs");
            let merge: bool = cmd.is_present("merge");
            let output: PathBuf = args.value_of("output")
                // No need to re-validate since we know the file is good, its folder must be too
                .map_or_else(
                    // If we do not get an output folder (or it's invalid), place the output alongside our input file
                    || file.parent().unwrap_or_else(|| Path::new("/")).into(),
                    // If we do, attempt to get a path from it
                    PathBuf::from
                );

            // We haven't validated the file here
            let version_file: &str = cmd.value_of("version-keys").unwrap();
            let version_keys: Option<Vec<version::Data>> = if key_1.is_none() || key_2.is_none() {
                    // Let's validate that the file exists
                    if let Err(e) = validate::is_file(version_file) {
                        eprintln!("Error opening version keys file : {}", e);
                        return;
                    }
                    match version::read_version_file(PathBuf::from(version_file)) {
                        Ok(keys) => Some(keys),
                        Err(e) => {
                            eprintln!("Error reading key file : {}", e);
                            return;
                        }
                    }
                } else { None };

            // Function call will be added later
            println!("FileDemux: file: {}, output: {}, key1: {:?}, key2: {:?}, version file: {:?}, subs: {}, merge: {}, cleanup: {}",
                file.to_str().unwrap(),
                output.to_str().unwrap(),
                key_1, key_2, version_file, subs, merge, cleanup);
            let _res = demux::process_file(file, version_keys, key_2, key_1, output.as_path());
        },
        Some(("batchDemux", cmd)) => {
            // Start to extract the arguments
            // Clap already validated the paths and the key values if any
            let folder: PathBuf = PathBuf::from(cmd.value_of("usm-folder").unwrap());
            let subs: bool = cmd.is_present("subs");
            let merge: bool = cmd.is_present("merge");
            let output: PathBuf = args.value_of("output")
                // No need to re-validate since we know the file is good, its folder must be too
                .map_or_else(
                    // If we do not get an output folder (or it's invalid), place the output alongside our input file
                    || folder.clone(),
                    // If we do, attempt to get a path from it
                    PathBuf::from
                );

            // We haven't validated this json, we need to. We just know it's a file that exists
            let version_keys: Vec<version::Data> = match version::read_version_file(cmd.value_of("version-keys").unwrap())
            {
                Ok(good) => good,
                Err(e) => {
                    eprintln!("Error opening version keys file: {}", e);
                    return;
                }
            };

            println!("FileDemux: folder: {}, output: {}, version file: {}, subs: {}, merge: {}, cleanup: {}",
                folder.to_str().unwrap(),
                output.to_str().unwrap(),
                version_keys.len(),
                subs, merge, cleanup);
        },
        Some(("convertHca", cmd)) => {
            // Start to extract the arguments
            // Clap already validated the paths and the key values if any
            let file: PathBuf = PathBuf::from(cmd.value_of("hca-input").unwrap());
            let output: PathBuf = args.value_of("output")
                // No need to re-validate since we know the file is good, its folder must be too
                .map_or_else(
                    // If we do not get an output folder (or it's invalid), place the output alongside our input file
                    || file.parent().unwrap_or_else(|| Path::new("/")).into(),
                    // If we do, attempt to get a path from it
                    PathBuf::from
                );

            println!("FileDemux: folder: {}, output: {}, cleanup: {}",
                file.to_str().unwrap(),
                output.to_str().unwrap(),
                cleanup);
        },
        _ => { eprintln!("No subcommand provided"); }
    }
}
