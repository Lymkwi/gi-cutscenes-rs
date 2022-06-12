# GI-cutscenes : Rust Remix

A command line program playing with the cutscene files (USM) from Genshin Impact, reimplemented in Rust.

Much like its [original C\# implementation by ToaHartor](https://github.com/ToaHartor/GI-cutscenes), it is able to demux USM files, decrypt video and audio tracks, convert HCA files to WAV, convert SRT subtitles into ASS and merge all of these files into a single MKV file.

(note: the subtitle system is not ready yet)

**In order to translate cutscenes of 2.7** check out [the `versions.json` file](https://raw.githubusercontent.com/ToaHartor/GI-cutscenes/main/versions.json) provided by ToaHartor.

This implementation depends on `ffmpeg` being installed on the system (for now). We are working on being able to merge the different formats ourselves, but this is not yet operational.

Commits are typically verified with GPG key [`E30568E404157F2A932071532C5FA04C19678729`](https://vulpinecitrus.info/Lymkwi.gpg.txt).
You can get it with
```bash
curl -L https://vulpinecitrus.info/Lymkwi.gpg.txt | gpg --import -
```
You can then verify release signatures using that key :
```bash
gpg --verify <release-file> <release-sig>.asc
```

## Usage

You can simply call the program and provide it top-level arguments, a subcommand, and sub-command arguments.

TL;DR at the bottom.

The available top-level arguments are :
 - `-o`/`--output` : Outfile file or folder
 - `-V`/`--version` : Print version information
 - `-h`/`--help` : Print help information
 - `--no-cleanup` : Do not remove the extracted files after conversion/merging

The subcommands are :
 - `demuxUsm` : Demux a single USM file. Its arguments are :
    - `-a`/`--key1` : the 4 lower bytes of the decryption key (hexadecimal)
    - `-b`/`--key2` : the 4 higher bytes of the encryption key (hexadecimal)
    - `-f`/`--demux-file` : Path to the file to be demuxed
    - `-k`/`--version-keys` : Path to the `versions.json` file (defaults to `./versions.json`)
    - `-m`/`--merge` : Flag to indicate we wish to merge the output IVF, WAV and ASS into a MKV container
    - `-p`/`--merge-program` : Path to the FFMpeg merge program (defaults to `ffmpeg`)
    - `-s`/`--subtitles` : Flag to indicate we want to add the subtitles to the MKV file
 - `batchDemux` : Demux a whole folder of USM files. Arguments are :
    - `-u`/`--usm-folder` : Path to the folder containing USM files
    - `-k`/`--version-keys` : Path to the `versions.json` file (defaults to `./versions.json`)
    - `-m`/`--merge` : Flag to indicate we wish to merge the output IVF, WAV and ASS into a MKV container
    - `-p`/`--merge-program` : Path to the FFMpeg merge program (defaults to `ffmpeg`)
    - `-s`/`--subtitles` : Flag to indicate we want to add the subtitles to the MKV file
 - `convertHca` : Convert a HCA file to WAV
    - `-i`/`--hca-input` : Path to the input HCA file
    - `-a`/`--key1` : the 4 lower bytes of the decryption key (hexadecimal)
    - `-b`/`--key2` : the 4 higher bytes of the encryption key (hexadecimal)
    - `-k`/`--version-keys` : Path to the `versions.json` file (defaults to `./versions.json`)
    - `-n`/`--base-name` : Base name to look for in the `versions.json` file to find keys

### TL;DR

Here are the most common commands :

**Demux a given cutscene**
```bash
./gi-cutscenes-rs -o battlePass.mkv demuxUsm -m -f battlePass.usm -k ./versions.json
```

**Demux a batch of cutscenes**
```bash
./gi-cutscenes-rs -o cutscene-output batchDemux -m -u usm-files/ -k versions.json
```

**Convert a HCA file to WAV**
```bash
./gi-cutscenes-rs -o battlePass_0.wav convertHca -n battlePass -i battlePass_0.hca
```

## Build & Install

This implementation is written in Rust for speed and efficiency. It can be built with `cargo` using :
```
cargo build --release
```

And installed in your cargo folder using :
```
cargo install --path .
```

### External Requirements

While we aim to be able to only depend on the standard Rust library, and a crate here and there (notably [`clap`](https://lib.rs/clap)),
there is no crate suitable to let us manually merge the IVF, WAV and ASS files into a MKV. As such, we depend on `ffmpeg` for this.

While the original C\# implementation of this tool relied on [MKVMerge](https://www.bunkus.org/blog/) from the MKVToolNix suite, we figured
`ffmpeg` would be better suited and more flexible.

It can be installed on
 - [Windows](https://www.gyan.dev/ffmpeg/builds/) using GyanDev's builds
 - [Linux](https://ffmpeg.org/download.html#build-linux) using your favourite package manager
 - [MacOS](https://ffmpeg.org/download.html#build-mac)
 - just about anything where you can [compile the source code](https://ffmpeg.org/download.html#get-sources) (I've heard people put it on embedded hardware, it's not *that* hard)

As soon as it is built, either the path to its binary is added in your path variable, or you can supply it using the `-p`/`--merge-program`
argument described above.

## Roadmap

 - [X] Full pipeline of USM to (HCA + IVF) to (WAV + IVF) to MKV
 - [X] Batch demux
 - [X] Single HCA to WAV file
 - [ ] Merging of sub files (obtainable in [Dimbreath's repository](https://github.com/Dimbreath/GenshinData/tree/master/Subtitle))

## License

Just as the original, this is released under GPL 3.0.

## Acknowledgements

I would like to thank [ToaHartor](https://github.com/ToaHartor) for his commitment to understanding the encryption system used in Genshin Impact's USM files, as well as the code this implementation is based on.
