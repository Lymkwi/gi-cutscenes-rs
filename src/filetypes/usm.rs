
// The default for everything here is 0 (or 0.0)
#[derive(Default)]
struct USMInfo {
    sig: u32,
    data_size: u32,
    data_offset: u8,
    padding_size: u16,
    chno: u8,
    data_type: u8,
    frame_time: u32,
    frame_rate: u32
}

#[allow(dead_code)]
pub struct USMFile {
    filename: String,
    path: PathBuf,
    key1: [u8; 4],
    key2: [u8; 4],
    video_mask_1: [u8; 32],
    video_mask_2: [u8; 32],
    audio_mask: [u8; 32]
}

impl USMFile {
    pub fn new(file: PathBuf, key2: [u8; 4], key1: [u8; 4]) -> Self {
        let mut res: Self = Self {
            filename: file.file_name().unwrap().to_str().unwrap().into(),
            path: file,
            key1,
            key2,
            video_mask_1: [0; 32],
            video_mask_2: [0; 32],
            audio_mask: [0; 32]
        };

        res.init_mask(key2, key1);

        res
    }

    fn init_mask(&mut self, key2: [u8; 4], key1: [u8; 4]) {
        self.video_mask_1[0x00] = key1[0];
        self.video_mask_1[0x01] = key1[1];
        self.video_mask_1[0x02] = key1[2];
        self.video_mask_1[0x03] = key1[3].wrapping_sub(0x34);
        self.video_mask_1[0x04] = key2[0].wrapping_add(0xF9);
        self.video_mask_1[0x05] = key2[1] ^ 0x13;
        self.video_mask_1[0x06] = key2[2].wrapping_add(0x61);
        self.video_mask_1[0x07] = self.video_mask_1[0x00] ^ 0xFF;
        self.video_mask_1[0x08] = self.video_mask_1[0x02].wrapping_add(self.video_mask_1[0x01]);
        self.video_mask_1[0x09] = self.video_mask_1[0x01].wrapping_sub(self.video_mask_1[0x07]);
        self.video_mask_1[0x0A] = self.video_mask_1[0x02] ^ 0xFF;
        self.video_mask_1[0x0B] = self.video_mask_1[0x01] ^ 0xFF;
        self.video_mask_1[0x0C] = self.video_mask_1[0x0B].wrapping_add(self.video_mask_1[0x09]);
        self.video_mask_1[0x0D] = self.video_mask_1[0x08].wrapping_sub(self.video_mask_1[0x03]);
        self.video_mask_1[0x0E] = self.video_mask_1[0x0D] ^ 0xFF;
        self.video_mask_1[0x0F] = self.video_mask_1[0x0A].wrapping_sub(self.video_mask_1[0x0B]);
        self.video_mask_1[0x10] = self.video_mask_1[0x08].wrapping_sub(self.video_mask_1[0x0F]);
        self.video_mask_1[0x11] = self.video_mask_1[0x10] ^ self.video_mask_1[0x07];
        self.video_mask_1[0x12] = self.video_mask_1[0x0F] ^ 0xFF;
        self.video_mask_1[0x13] = self.video_mask_1[0x03] ^ 0x10;
        self.video_mask_1[0x14] = self.video_mask_1[0x04].wrapping_sub(0x32);
        self.video_mask_1[0x15] = self.video_mask_1[0x05].wrapping_add(0xED);
        self.video_mask_1[0x16] = self.video_mask_1[0x06] ^ 0xF3;
        self.video_mask_1[0x17] = self.video_mask_1[0x13].wrapping_sub(self.video_mask_1[0x0F]);
        self.video_mask_1[0x18] = self.video_mask_1[0x15].wrapping_add(self.video_mask_1[0x07]);
        self.video_mask_1[0x19] = 0x21_u8.wrapping_sub(self.video_mask_1[0x13]);
        self.video_mask_1[0x1A] = self.video_mask_1[0x14] ^ self.video_mask_1[0x17];
        self.video_mask_1[0x1B] = self.video_mask_1[0x16].wrapping_add(self.video_mask_1[0x16]);
        self.video_mask_1[0x1C] = self.video_mask_1[0x17].wrapping_add(0x44);
        self.video_mask_1[0x1D] = self.video_mask_1[0x03].wrapping_add(self.video_mask_1[0x04]);
        self.video_mask_1[0x1E] = self.video_mask_1[0x05].wrapping_sub(self.video_mask_1[0x16]);
        self.video_mask_1[0x1F] = self.video_mask_1[0x1D] ^ self.video_mask_1[0x13];

        let lookup: [u8; 4] = [ 85, 82, 85, 67 ]; // "URUC"
        for i in 0..0x20 {
            self.video_mask_2[i] = self.video_mask_1[i] ^ 0xFF;
            self.audio_mask[i] = if i % 2 == 1 { lookup[i >> 1 & 3] } else { self.video_mask_1[i] ^ 0xFF };
        }
    }

    fn mask_video(&mut self, data: &mut [u8], size: usize) {
        let data_offset = 0x40;
        if size < data_offset { return; }
        let size = size - data_offset;
        if size >= 0x200 {
            let mut mask: [u8; 0x20] = self.video_mask_2;
            for i in 0x100..size {
                data[i + data_offset] ^= mask[i & 0x1F];
                mask[i & 0x1F] = data[i + data_offset] ^ self.video_mask_2[i & 0x1F];
            }
            let mut mask: [u8; 0x20] = self.video_mask_1;
            for i in 0..0x100 {
                mask[i & 0x1F] ^= data[0x100 + i + data_offset];
                data[i + data_offset] ^= mask[i & 0x1F];
            }
        }
    }
}

impl Demuxable for USMFile {
    fn demux(mut self, video_extract: bool, audio_extract: bool, output: &Path) -> GICSResult<(PathBuf, Vec<PathBuf>)> {
        let f = File::open(self.path.as_path())?;
        let mut file_size = f.metadata()?.len();
        let mut reader = BufReader::new(f);
        let mut info: USMInfo = USMInfo::default();

        // Base output folder
        let base_output = PathBuf::from(output);
        // Try and create it
        if !base_output.is_dir() {
            // We check with is dir because if the path exists and is a file/socket/etc
            // it's also going to crash and be handled here
            std::fs::create_dir_all(base_output.as_path())?;
        }

        // Base file name
        let base_name = self.path.file_name().ok_or_else(|| GICSError::new("USM Path has no file name"))?;

        // Video output
        let mut video_path = base_output.clone();
        video_path.push(base_name);
        video_path.set_extension("ivf");
        let mut video_output = BufWriter::new(File::create(video_path.as_path())?);

        // Audio outputs
        let audio_path = base_output.clone();
        // The extension is set later
        let audio_base_name: String = self.path
            .file_stem().ok_or_else(|| GICSError::new("USM path has no file stem"))?
            .to_str().ok_or_else(|| GICSError::new("Unable to decode USM file stem to UTF-8"))?
            .into();
        let mut audio_writers: HashMap<u8, BufWriter<_>> = HashMap::new();
        let mut audio_files: Vec<PathBuf> = Vec::new();

        while file_size > 0 {
            // Read 32 bits at a time
            let mut byte_block: [u8; 0x20] = [0; 0x20];
            reader.read_exact(&mut byte_block)?;
            file_size -= 32;

            // Parse info from the content
            info.sig = make_be32(&byte_block[0..4]);
            info.data_size = make_be32(&byte_block[4..8]);
            info.data_offset = byte_block[9];
            info.padding_size = make_be16(&byte_block[10..12]);
            info.chno = byte_block[12];
            info.data_type = byte_block[15];
            info.frame_time = make_be32(&byte_block[16..20]);
            info.frame_rate = make_be32(&byte_block[20..24]);

            // Now work with the rest of the data
            // Read the size of the data
            let size: usize = (info.data_size - u32::from(info.data_offset) - u32::from(info.padding_size)) as usize;
            reader.seek(SeekFrom::Current(i64::from(info.data_offset - 0x18)))?;
            let mut data = vec![0u8; size];
            reader.read_exact(&mut data)?;
            // Skip padding
            reader.seek(SeekFrom::Current(i64::from(info.padding_size)))?;
            // Account for it
            file_size -= u64::from(info.data_size - 0x18);

            // Depending on the signature, do something different
            match info.sig {
                //0x4352_4944 => { /* (CRID) Nothing to do */ },
                0x4053_4656 => {
                    // It's a video block (@SFV)
                   if info.data_type == 0 && video_extract {
                        self.mask_video(&mut data, size as usize);
                        video_output.write_all(&data)?;
                    }
                },
                0x4053_4641 => {
                    if info.data_type == 0  && audio_extract {
                        // It's an audio block (@SFA)
                        // FIXME: Do we extract audio ?
                        if !audio_writers.contains_key(&info.chno) {
                        }
                        if let std::collections::hash_map::Entry::Vacant(e) = audio_writers.entry(info.chno) {
                            let filename = format!("{}_{}.hca", audio_base_name, info.chno);
                            let mut path = audio_path.clone();
                            path.push(filename);
                            audio_files.push(path.clone());
                            e.insert(BufWriter::new(File::create(path)?));
                        }
                        audio_writers.get_mut(&info.chno).unwrap().write_all(&data)?;
                    }
                }
                _ => { /* we don't care */}
            }
        }
        Ok((video_path, audio_files))
    }
}