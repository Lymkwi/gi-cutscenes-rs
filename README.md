# GI-cutscenes : Rust Remix

A command line program playing with the cutscene files (USM) from Genshin Impact, reimplemented in Rust.

Much like its [original C\# implementation by ToaHartor](https://github.com/ToaHartor/GI-cutscenes), it is able to demux USM files, decrypt video and audio tracks, convert HCA files to WAV, convert SRT subtitles into ASS and merge all of these files into a single MKV file.

## Usage

\[TODO\]

## Build

This implementation is written in Rust for speed and efficiency. It can be built with `cargo` using :
```
cargo build --release
```

And installed in your cargo folder using :
```
cargo install --path .
```

## License

Just as the original, this is released under GPL 3.0.

## Acknowledgements

I would like to thank [ToaHartor](https://github.com/ToaHartor) for his commitment to understanding the encryption system used in Genshin Impact's USM files, as well as the code this implementation is based on.
