struct HCAHeader {
    version: u16,
    data_offset: u16,
    channel_count: u16,
    sampling_rate: u32,
    block_count: u32,
    block_size: u16,

    comp_r01: u32,
    comp_r02: u32,
    comp_r03: u32,
    comp_r04: u32,
    comp_r05: u32,
    comp_r06: u32,
    comp_r07: u32,
    comp_r08: u32,
    comp_r09: u32,

    ath_type: u16,

    loop_flag: u16,

    cipher_type: u16,

    volume: f32
}

impl Default for HCAHeader {
    fn default() -> Self {
        Self {
            version: 0,
            data_offset: 0,
            channel_count: 0,
            sampling_rate: 0,
            block_count: 0,
            block_size: 0,

            comp_r01: 0,
            comp_r02: 0,
            comp_r03: 0,
            comp_r04: 0,
            comp_r05: 0,
            comp_r06: 0,
            comp_r07: 0,
            comp_r08: 0,
            comp_r09: 0,

            ath_type: 0,
            loop_flag: 0,
            cipher_type: 0,
            volume: 0.0
        }
    }
}

pub struct HCAFile {
    filename: PathBuf,
    key1: [u8; 4],
    key2: [u8; 4],
    cipher_table: [u8; 0x100],
    ath_table: [u8; 0x100], // ?
    encrypted: bool,
    hca_header: HCAHeader,
    hca_channel: Vec<Channel>,
    header: Vec<u8>,
    data: Vec<u8>
}

impl HCAFile {
    pub fn new(path: PathBuf, key1: [u8; 4], key2: [u8; 4]) -> Result<Self> {
        let mut res = Self {
            filename: path,
            key1, key2,
            cipher_table: [0; 0x100],
            ath_table: [0; 0x100],
            encrypted: false,
            hca_header: HCAHeader::default(),
            hca_channel: Vec::new(),
            header: Vec::new(),
            data: Vec::new()
        };
        res.read_header()?;
        Ok(res)
    }

    fn read_header(&mut self) -> Result<Vec<u8>> {
        if !self.filename.exists() {
            return Err(Error::new(ErrorKind::NotFound, "Could not find file"));
        }
        if self.filename.extension().unwrap().to_str().unwrap() != "hca" {
            return Err(Error::new(ErrorKind::InvalidData, "File extension isn't HCA"));
        }
        let mut fs: File = File::open(&self.filename)?;

        let mut hca_byte: [u8; 8] = [0; 8];
        fs.read_exact(&mut hca_byte)?;

        let mut sign = u32::from_be_bytes([hca_byte[0], hca_byte[1], hca_byte[2], hca_byte[3]]) & 0x7F7F7F7F;
        let magic = if sign == 0x00144348 {
            self.encrypted = true;
            0x7F7F7F7F
        } else {
            0xFFFFFFFF
        };

        sign = u32::from_be_bytes([hca_byte[0], hca_byte[1], hca_byte[2], hca_byte[3]]) & magic;
        if sign == 0x00414348 {
            let conv = sign.to_le_bytes();
            (0..4).for_each(|i| hca_byte[i] = conv[i]);
            self.hca_header.version = u16::from_be_bytes([hca_byte[4], hca_byte[5]]);
            self.hca_header.data_offset = u16::from_be_bytes([hca_byte[6], hca_byte[7]]);
        } else {
            panic!("Wrong header!");
        }

        let mut header: Vec<u8> = vec![0; usize::from(self.hca_header.data_offset)];
        fs.seek(SeekFrom::Start(0))?;
        fs.read_exact(&mut header)?;
        header.iter_mut().zip(&hca_byte).for_each(|(dest, source)| *dest = *source);
        let mut header_offset: usize = 8;

        // Format
        sign = u32::from_le_bytes([
            header[header_offset],
            header[header_offset+1],
            header[header_offset+2],
            header[header_offset+3]
        ]) & magic;

        if sign == 0x00746D66 {
            header[header_offset .. header_offset + 4].iter_mut().zip(&hca_byte).for_each(|(dest, source)| *dest = *source);
            //header[header_offset] = converted[0];
            self.hca_header.channel_count = u16::from(header[header_offset + 4]);
            let mut sampling_rate: [u8; 4] = [0; 4];
            (&mut sampling_rate[1..4]).iter_mut().zip(&header[header_offset + 5 .. header_offset + 8]).for_each(|(dest, source)| *dest = *source);
            self.hca_header.sampling_rate = u32::from_be_bytes(sampling_rate);
            self.hca_header.block_count = u32::from_be_bytes([
                header[header_offset + 8],
                header[header_offset + 9],
                header[header_offset + 0xA],
                header[header_offset + 0xB]
            ]);
            header_offset += 16;
        } else {
            panic!("Broken FMT header");
        }

        sign = u32::from_le_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
    
        if sign == 0x706D6F63 { // COMP
            header.iter_mut().zip(&sign.to_le_bytes()).for_each(|(dest, source)| *dest = *source);
            self.hca_header.block_size = u16::from_be_bytes([
                header[header_offset + 4],
                header[header_offset + 5]
            ]);
            self.hca_header.comp_r01 = u32::from(header[header_offset + 0x6]);
            self.hca_header.comp_r02 = u32::from(header[header_offset + 0x7]);
            self.hca_header.comp_r03 = u32::from(header[header_offset + 0x8]);
            self.hca_header.comp_r04 = u32::from(header[header_offset + 0x9]);
            self.hca_header.comp_r05 = u32::from(header[header_offset + 0xA]);
            self.hca_header.comp_r06 = u32::from(header[header_offset + 0xB]);
            self.hca_header.comp_r07 = u32::from(header[header_offset + 0xC]);
            self.hca_header.comp_r08 = u32::from(header[header_offset + 0xD]);

            if !(self.hca_header.block_size >= 8 || self.hca_header.block_size == 0) {
                panic!("Invalid block size");
            }
            if !(self.hca_header.comp_r01 <= self.hca_header.comp_r02 && self.hca_header.comp_r02 <= 0x1F) {
                panic!("Incorrect comp values");
            }
            header_offset += 16;
        } else if sign == 0x00636564 {
            header.iter_mut().zip(&sign.to_le_bytes()).for_each(|(dest, source)| *dest = *source);
            self.hca_header.block_size = u16::from_be_bytes([
                header[header_offset + 4],
                header[header_offset + 5]
            ]);
            self.hca_header.comp_r01 = u32::from(header[header_offset + 0x6]);
            self.hca_header.comp_r02 = u32::from(header[header_offset + 0x7]);
            self.hca_header.comp_r03 = u32::from(header[header_offset + 0xA] >> 4);
            self.hca_header.comp_r04 = u32::from(header[header_offset + 0xA] & 0xF);
            self.hca_header.comp_r05 = u32::from(header[header_offset + 0x8]);
            self.hca_header.comp_r06 = if header[header_offset + 0xB] > 0 {
                u32::from(header[header_offset + 0x9])
            } else {
                u32::from(header[header_offset + 0x8] + 1)
            };
            self.hca_header.comp_r07 = u32::from(self.hca_header.comp_r05 - self.hca_header.comp_r06);
            self.hca_header.comp_r08 = 0;
            if !(self.hca_header.block_size >= 8 || self.hca_header.block_size == 0) {
                panic!("Invalid block_size");
            }
            if !(self.hca_header.comp_r01 <= self.hca_header.comp_r02 && self.hca_header.comp_r02 <= 0x1F) {
                panic!("Invalid comp values");
            }
            if self.hca_header.comp_r03 == 0 { self.hca_header.comp_r01 = 1; }
            header_offset += 12;
        } else {
            panic!("Invalid field");
        }


        Ok(Vec::new())
    }

    pub fn convert_to_wav(mut self, path: &Path) -> Result<()> {
        // Some definitions
        let volume = 1.0;
        let mode = 16;
        let loop_flag = 0;

        let mut wav_riff = WaveRiff::default();
        wav_riff.fmt_type = if mode > 0 { 1 } else { 3 };
        wav_riff.fmt_channel_count = self.hca_header.channel_count;
        wav_riff.fmt_bit_count = if mode > 0 { mode } else { 32 };
        wav_riff.fmt_sampling_rate = self.hca_header.sampling_rate;
        wav_riff.fmt_sampling_size = wav_riff.fmt_bit_count / 8 * wav_riff.fmt_channel_count;

        // Fill in wav sample
        let wav_simp = WaveSample::default();
        let mut wav_data = WaveData::default();
        wav_data.data_size = self.hca_header.block_count * 0x80 * 8 * u32::from(wav_riff.fmt_sampling_size) + (wav_simp.loop_end - wav_simp.loop_start) * loop_flag;
        wav_riff.riff_size = 0x1C + 8 + wav_data.data_size; // 8 is std::mem::size_of::<WaveData>()

        // We do not consider wave samples here, Genshin does not need to have any for the HCA to WAV conversion
        let mut header: Vec<u8> = Vec::new();
        header.extend(wav_riff.build_byte_array());
        header.extend(wav_data.build_byte_array());

        // Build a path to the wav file
        let mut wav_path = PathBuf::from(path); // ugly clone
        wav_path.set_extension("wav");
        println!("Writing to {}", wav_path.to_str().unwrap());

        // Start to write the actual wav file
        let mut wav_file: File = File::create(wav_path)?;
        wav_file.write(&header)?;

        self.hca_header.volume *= volume;
    
        let block_size: usize = self.hca_header.block_size.try_into().unwrap();
        let block_count: usize = self.hca_header.block_count.try_into().unwrap();
        let mut data_2: Vec<u8> = vec![0_u8; block_size];
        let mut offset: usize = 0;

        for _ in 0..block_count {
            // Copy <block size> bytes of data starting at offset into data 2
            data_2.iter_mut().zip(&self.data[offset .. (offset + block_size)]).for_each(|(target, source)| *target = *source);
            self.decode_block(&mut data_2);

            for i in 0..8 {
                for j in 0..0x80 {
                    for k in 0..self.hca_header.channel_count {
                        let mut f = f64::from(self.hca_channel[usize::from(k)].wave[i][j] * self.hca_header.volume);
                        // Don't saturate
                        if f > 1.0 { f = 1.0; }
                        else if f < -1.0 { f = -1.0; }
                        // Decoding mode
                        let v = match mode {
                            0x00 => f.trunc(), // float mode
                            0x08 => (f * f64::from(i8::MAX)).trunc() + 128.0, // 8 bit mode
                            0x10 => (f * f64::from(i16::MAX)).trunc(), // 16 bits
                            0x18 => (f * f64::from(0x800000 - 1)).trunc(), // 24 bits
                            0x20 => (f * f64::from(i32::MAX)).trunc(), // 32 bits
                            _ => {
                                panic!("Mode not supported: {}", mode);
                            }
                        } as i32;
                        wav_file.write(&v.to_be_bytes())?;
                    }
                }
            }
            offset += usize::from(self.hca_header.block_size);
        }

        // Flush and close
        wav_file.flush()?;

        Ok(())
    }

    fn decode_block(&mut self, data: &mut [u8]) {
        self.mask(data, self.hca_header.block_size as usize);
        let mut data_block: ClData = ClData::new(data.iter().map(|&u| i32::from(u)).collect(), i32::from(self.hca_header.block_size));
        let magic = data_block.get_bit(16);
        if magic != 0xFFFF {
            return;
        }
        let a = (data_block.get_bit(9) << 8) - data_block.get_bit(7);

        let channel_count = self.hca_header.channel_count;
        (0..channel_count).for_each(|i| {
            self.hca_channel[i as usize].decode_one(&mut data_block, self.hca_header.comp_r09, a, self.ath_table.to_vec());
        });

        // Do 8 rounds of decoding
        (0..8).for_each(|i| {
            (0..channel_count).for_each(|j| self.hca_channel[usize::from(j)].decode_two(&mut data_block));
            (0..channel_count).for_each(|j|
                self.hca_channel[usize::from(j)]
                    .decode_three(self.hca_header.comp_r09, self.hca_header.comp_r08,
                        self.hca_header.comp_r07 + self.hca_header.comp_r06,
                        self.hca_header.comp_r05
                    )
            );
            (0..channel_count).for_each(|j| {
                //let chan = &mut self.hca_channel[1];
                // I have to return here because I can't borrow the array of channels and
                // The first channel as mutable at once
                let former_channel = self.hca_channel[1];
                let decoded_channel = self.hca_channel[usize::from(j)]
                    .decode_four(i, self.hca_header.comp_r05 - self.hca_header.comp_r06,
                            self.hca_header.comp_r06, self.hca_header.comp_r07, &former_channel);
                self.hca_channel[1] = decoded_channel;
            });
            (0..channel_count).for_each(|j|
                self.hca_channel[usize::from(j)].decode_five(i)
            );
        });
    }

    fn mask(&self, data: &mut [u8], block_size: usize) {
        (0..block_size).for_each(|i| {
            let d = data[i];
            data[i] = self.cipher_table[usize::from(d)];
        })
    }
}