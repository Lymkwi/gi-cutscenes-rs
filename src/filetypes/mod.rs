use std::io::{
    prelude::*,
    BufReader,
    BufWriter,
    Error,
    ErrorKind,
    Result,
    SeekFrom
};

use std::collections::HashMap;
use std::fs::File;
use std::path::{
    Path,
    PathBuf
};

use crate::{
    demux::Demuxable,
    tools::{
        make_be32,
        make_be16
    }
};

include!("usm.rs");
include!("channel.rs");
include!("hca.rs");
include!("wav.rs");