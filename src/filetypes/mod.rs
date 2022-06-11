use std::{
    collections::HashMap,
    fs::File,
    io::{
        prelude::*,
        BufReader,
        BufWriter,
        SeekFrom
    },
    path::{
        Path,
        PathBuf
    },
    process::Command
};

use crate::{
    demux::Demuxable,
    errors::{
        GICSError,
        GICSResult
    },
    tools::{
        make_be32,
        make_be16
    }
};

include!("channel.rs");
include!("hca.rs");
include!("mkv.rs");
include!("usm.rs");
include!("wav.rs");