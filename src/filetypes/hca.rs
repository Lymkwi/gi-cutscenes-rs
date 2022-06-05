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

    loop_flag: bool,

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
            loop_flag: false,
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
    ath_table: [u8; 0x80],
    encrypted: bool,
    hca_header: HCAHeader,
    hca_channel: Vec<Channel>,
    header: Vec<u8>,
    data: Vec<u8>
}

impl HCAFile {
    pub fn new(path: PathBuf, key2: [u8; 4], key1: [u8; 4]) -> GICSResult<Self> {
        let mut res = Self {
            filename: path,
            key1, key2,
            cipher_table: [0; 0x100],
            ath_table: [0; 0x80],
            encrypted: false,
            hca_header: HCAHeader::default(),
            hca_channel: Vec::new(),
            header: Vec::new(),
            data: Vec::new()
        };
        res.read_header()?;
        Ok(res)
    }

    #[allow(clippy::too_many_lines)]
    fn read_header(&mut self) -> GICSResult<()> {
        if !self.filename.exists() {
            return Err(GICSError::new("Could not find file"));
        }
        if self.filename.extension().unwrap().to_str().unwrap() != "hca" {
            return Err(GICSError::new("File extension isn't HCA"));
        }
        let mut fs: File = File::open(&self.filename)?;

        let mut hca_byte: [u8; 8] = [0; 8];
        fs.read_exact(&mut hca_byte)?;

        let mut sign = u32::from_le_bytes([hca_byte[0], hca_byte[1], hca_byte[2], hca_byte[3]]) & 0x7F7F_7F7F;
        let magic = if sign == 0x0041_4348 {
            self.encrypted = true;
            0x7F7F_7F7F
        } else {
            0xFFFF_FFFF
        };

        sign = u32::from_le_bytes([hca_byte[0], hca_byte[1], hca_byte[2], hca_byte[3]]) & magic;
        if sign == 0x0041_4348 {
            let conv = sign.to_le_bytes();
            (0..4).for_each(|i| hca_byte[i] = conv[i]);
            self.hca_header.version = u16::from_be_bytes([hca_byte[4], hca_byte[5]]);
            self.hca_header.data_offset = u16::from_be_bytes([hca_byte[6], hca_byte[7]]);
        } else {
            return Err(GICSError::new("unknown signature for version and data_offset block"));
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

        if sign == 0x0074_6D66 {
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
            return Err(GICSError::new("Broken FMT header"));
        }

        sign = u32::from_le_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
    
        if sign == 0x706D_6F63 { // COMP
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
                return Err(GICSError::new("invalid block size during HCA header reading"));
            }
            if !(self.hca_header.comp_r01 <= self.hca_header.comp_r02 && self.hca_header.comp_r02 <= 0x1F) {
                return Err(GICSError::new("invalid comp register (1 and 2) values found during HCA header reading"));
            }
            header_offset += 16;
        } else if sign == 0x0063_6564 {
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
            self.hca_header.comp_r07 = self.hca_header.comp_r05 - self.hca_header.comp_r06;
            self.hca_header.comp_r08 = 0;
            if !(self.hca_header.block_size >= 8 || self.hca_header.block_size == 0) {
                return Err(GICSError::new("invalid block size found during HCA header reading"));
            }
            if !(self.hca_header.comp_r01 <= self.hca_header.comp_r02 && self.hca_header.comp_r02 <= 0x1F) {
                return Err(GICSError::new("invalid comp register (1 and 2) values found during HCA header reading"));
            }
            if self.hca_header.comp_r03 == 0 { self.hca_header.comp_r01 = 1; }
            header_offset += 12;
        } else {
            return Err(GICSError::new("Invalid field"));
        }


        // VBR
        sign = u32::from_le_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
        if sign == 0x0072_6276 {
            header.iter_mut().zip(sign.to_le_bytes()).for_each(|(dst, src)| *dst = src);
            header_offset += 8;
        }

        // ATH
        sign = u32::from_be_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
        if sign == 0x6174_6800 {
            header.iter_mut().zip(sign.to_be_bytes()).for_each(|(dst, src)| *dst = src);
            self.hca_header.ath_type = u16::from_be_bytes([header[header_offset + 4], header[header_offset + 4]]);
            header_offset += 6;
        } else if self.hca_header.version < 0x200 {
            self.hca_header.ath_type = 1;
        }

        // LOOP
        sign = u32::from_be_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
        if sign == 0x6C6F_6F70 {
            header.iter_mut().zip(sign.to_be_bytes()).for_each(|(dst, src)| *dst = src);
            self.hca_header.loop_flag = true;
            header_offset += 16;
        } else {
            self.hca_header.loop_flag = false;
        }

        // Cipher
        sign = u32::from_be_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
        if sign == 0x6369_7068 {
            header.iter_mut().zip(sign.to_be_bytes()).for_each(|(dst, src)| *dst = src);
            self.hca_header.cipher_type = u16::from_be_bytes([
                header[header_offset + 4], header[header_offset + 5]
            ]);
            if !(self.hca_header.cipher_type == 0 || self.hca_header.cipher_type == 1 || self.hca_header.cipher_type == 0x38) {
                return Err(GICSError::new("Invalid cipher type found during HCA header reading"));
            }
        } else {
            self.hca_header.cipher_type = 0;
        }

        // RVA
        sign = u32::from_be_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
        if sign == 0x7276_6100 {
            header.iter_mut().zip(sign.to_be_bytes()).for_each(|(dst, src)| *dst = src);
            self.hca_header.volume = f32::from_be_bytes([
                header[header_offset + 4], header[header_offset + 5],
                header[header_offset + 6], header[header_offset + 7]
            ]);
            header_offset += 8;
        } else {
            self.hca_header.volume = 1.0;
        }

        sign = u32::from_be_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
        if sign == 0x636F_6D6D {
            header.iter_mut().zip(sign.to_be_bytes()).for_each(|(dst, src)| *dst = src);
            header_offset += 5;
        }

        sign = u32::from_be_bytes([
            header[header_offset], header[header_offset + 1],
            header[header_offset + 2], header[header_offset + 3]
        ]) & magic;
        if sign == 0x7061_6400 {
            header.iter_mut().zip(sign.to_be_bytes()).for_each(|(dst, src)| *dst = src);
            //header_offset += 4;
        }

        let csum = Self::checksum(&header, header.len() - 2);
        header.iter_mut().zip(csum.to_be_bytes()).for_each(|(dst, src)| *dst = src);
        let data_size: usize = usize::from(self.hca_header.block_size) * self.hca_header.block_count as usize;
        self.data = vec![0; data_size];
        fs.read_exact(&mut self.data)?;

        self.ath_init()?;
        self.init_mask(self.hca_header.cipher_type);

        if self.hca_header.comp_r03 == 0 {
            self.hca_header.comp_r03 = 1;
        }

        self.channel_init()?;

        self.header.clear();
        self.header.extend(header);
        Ok(())
    }

    const fn reformulate(a: u32, b: u32) -> u32 {
        if b > 0 {
            a / b + (if a % b == 0 { 0 } else { 1 })
        } else {
            0
        }
    }

    fn channel_init(&mut self) -> GICSResult<()> {
        self.hca_channel = Vec::new();
        for _ in 0..self.hca_header.channel_count {
            self.hca_channel.push(Channel::new());
        }

        if !(self.hca_header.comp_r01 == 1 && self.hca_header.comp_r02 == 15) {
            return Err(GICSError::new("Comp register values 1 and 2 invalid for channel initialization"));
        }

        self.hca_header.comp_r09 = Self::reformulate(
            self.hca_header.comp_r05 - (self.hca_header.comp_r06 + self.hca_header.comp_r07),
            self.hca_header.comp_r08
        );

        let mut r: [u8; 0x10] = [0; 0x10];
        let b = u32::from(self.hca_header.channel_count) / self.hca_header.comp_r03;

        if self.hca_header.comp_r07 != 0 && b > 1 {
            let mut c = 0;
            for _ in 0..self.hca_header.comp_r03 {
                match b {
                    2 | 3 => {
                        r[c] = 1;
                        r[c + 1] = 2;
                    },
                    4 => {
                        r[c] = 1;
                        r[c + 1] = 2;
                        if self.hca_header.comp_r04 == 0 {
                            r[c + 2] = 1;
                            r[c + 3] = 2;
                        }
                    },
                    5 => {
                        r[c] = 1;
                        r[c + 1] = 2;
                        if self.hca_header.comp_r04 <= 2 {
                            r[c + 3] = 1;
                            r[c + 4] = 2;
                        }
                    },
                    6 | 7 => {
                        r[c] = 1;
                        r[c + 1] = 2;
                        r[c + 4] = 1;
                        r[c + 5] = 2;
                    },
                    8 => {
                        r[c] = 1;
                        r[c + 1] = 2;
                        r[c + 4] = 1;
                        r[c + 5] = 2;
                        r[c + 6] = 1;
                        r[c + 7] = 2;
                    },
                    _ => {}
                }
                c += b as usize;
            }
        }
        for i in 0..self.hca_header.channel_count {
            let index = usize::from(i);
            self.hca_channel[index].r#type = i32::from(r[index]);
            self.hca_channel[index].value_3i = self.hca_header.comp_r06 + self.hca_header.comp_r07;
            self.hca_channel[index].count = self.hca_header.comp_r06 + (if r[index] == 2 { 0 } else { self.hca_header.comp_r07 });
        }
        Ok(())
    }

    fn init_mask(&mut self, tp: u16) {
        match tp {
            0 => (0..0x100).for_each(|i| self.cipher_table[i] = i.try_into().unwrap()),
            1 => {
                let mut v: u8 = 0;
                for i in 0..0xFF {
                    v = v.wrapping_mul(13).wrapping_add(11);
                    // Redo if you're on the boundary
                    if v == 0 || v == 0xFF {
                        v = v.wrapping_mul(13).wrapping_add(11);
                    }
                    self.cipher_table[i] = v;
                }
                self.cipher_table[0] = 0;
                self.cipher_table[0xFF] = 0xFF;
            },
            56 => {
                let mut t1: [u8; 8] =  [0; 8];
                let mut key1: u32 = u32::from_le_bytes(self.key1);
                let mut key2: u32 = u32::from_le_bytes(self.key2);

                if key1 == 0 {
                    key2 -= 1;
                }
                key1 -= 1;

                for item in &mut t1 {
                    *item = key1.to_le_bytes()[0];
                    key1 = key1 >> 8 | key2 << 24;
                    key2 >>= 8;
                }

                let t2 = [
                    t1[1], t1[1] ^ t1[6], t1[2] ^ t1[3],
                    t1[2], t1[2] ^ t1[1], t1[3] ^ t1[4],
                    t1[3], t1[3] ^ t1[2], t1[4] ^ t1[5],
                    t1[4], t1[4] ^ t1[3], t1[5] ^ t1[6],
                    t1[5], t1[5] ^ t1[4], t1[6] ^ t1[1],
                    t1[6]                    
                ];
                let mut t3 = [0; 0x100];
                let t31: [u8; 0x10] = Self::init_cipher56_table(t1[0]);
                let mut t32: [u8; 0x10];
                for i in 0..0x10 {
                    t32 = Self::init_cipher56_table(t2[i]);
                    let v = t31[i] << 4;
                    t32.iter().enumerate().for_each(|(index, j)| {
                        t3[i * 0x10 + index] = v | j;
                    });
                }

                let mut i_table = 1;
                let mut v: usize = 0;
                for _ in 0..0x100 {
                    v = v.wrapping_add(0x11) & 0xFF;
                    let a = t3[v];
                    if a != 0 && a != 0xFF {
                        self.cipher_table[i_table] = a;
                        i_table += 1;
                    }
                }

                self.cipher_table[0] = 0;
                self.cipher_table[0xFF] = 0xFF;
            },
            _ => { }
        }
    }

    fn init_cipher56_table(val: u8) -> [u8; 0x10] {
        let mut table: [u8; 0x10] = [0; 0x10];
        let mul = (val & 1) << 3 | 5;
        let add = val & 0xE | 1;
        let mut key = val >> 4;
        for item in &mut table {
            key = key.wrapping_mul(mul).wrapping_add(add) & 0xF;
            *item = key;
        }
        table
    }

    fn ath_init(&mut self) -> GICSResult<()> {
        match self.hca_header.ath_type {
            0 => {
                self.ath_table = [0; 0x80];
                Ok(())
            },
            1 => {
                let list: [u8; 656] = [
                    0x78, 0x5F, 0x56, 0x51, 0x4E, 0x4C, 0x4B, 0x49, 0x48, 0x48, 0x47, 0x46, 0x46, 0x45, 0x45, 0x45,
                    0x44, 0x44, 0x44, 0x44, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
                    0x42, 0x42, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x40, 0x40, 0x40, 0x40,
                    0x40, 0x40, 0x40, 0x40, 0x40, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F,
                    0x3F, 0x3F, 0x3F, 0x3E, 0x3E, 0x3E, 0x3E, 0x3E, 0x3E, 0x3D, 0x3D, 0x3D, 0x3D, 0x3D, 0x3D, 0x3D,
                    0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B,
                    0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B,
                    0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3B, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C,
                    0x3D, 0x3D, 0x3D, 0x3D, 0x3D, 0x3D, 0x3D, 0x3D, 0x3E, 0x3E, 0x3E, 0x3E, 0x3E, 0x3E, 0x3E, 0x3F,
                    0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F, 0x3F,
                    0x3F, 0x3F, 0x3F, 0x3F, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
                    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                    0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                    0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
                    0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x43, 0x43, 0x43,
                    0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x43, 0x44, 0x44,
                    0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x44, 0x45, 0x45, 0x45, 0x45,
                    0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x46, 0x46, 0x46, 0x46, 0x46, 0x46, 0x46, 0x46,
                    0x46, 0x46, 0x47, 0x47, 0x47, 0x47, 0x47, 0x47, 0x47, 0x47, 0x47, 0x47, 0x48, 0x48, 0x48, 0x48,
                    0x48, 0x48, 0x48, 0x48, 0x49, 0x49, 0x49, 0x49, 0x49, 0x49, 0x49, 0x49, 0x4A, 0x4A, 0x4A, 0x4A,
                    0x4A, 0x4A, 0x4A, 0x4A, 0x4B, 0x4B, 0x4B, 0x4B, 0x4B, 0x4B, 0x4B, 0x4C, 0x4C, 0x4C, 0x4C, 0x4C,
                    0x4C, 0x4D, 0x4D, 0x4D, 0x4D, 0x4D, 0x4D, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4E, 0x4F, 0x4F, 0x4F,
                    0x4F, 0x4F, 0x4F, 0x50, 0x50, 0x50, 0x50, 0x50, 0x51, 0x51, 0x51, 0x51, 0x51, 0x52, 0x52, 0x52,
                    0x52, 0x52, 0x53, 0x53, 0x53, 0x53, 0x54, 0x54, 0x54, 0x54, 0x54, 0x55, 0x55, 0x55, 0x55, 0x56,
                    0x56, 0x56, 0x56, 0x57, 0x57, 0x57, 0x57, 0x57, 0x58, 0x58, 0x58, 0x59, 0x59, 0x59, 0x59, 0x5A,
                    0x5A, 0x5A, 0x5A, 0x5B, 0x5B, 0x5B, 0x5B, 0x5C, 0x5C, 0x5C, 0x5D, 0x5D, 0x5D, 0x5D, 0x5E, 0x5E,
                    0x5E, 0x5F, 0x5F, 0x5F, 0x60, 0x60, 0x60, 0x61, 0x61, 0x61, 0x61, 0x62, 0x62, 0x62, 0x63, 0x63,
                    0x63, 0x64, 0x64, 0x64, 0x65, 0x65, 0x66, 0x66, 0x66, 0x67, 0x67, 0x67, 0x68, 0x68, 0x68, 0x69,
                    0x69, 0x6A, 0x6A, 0x6A, 0x6B, 0x6B, 0x6B, 0x6C, 0x6C, 0x6D, 0x6D, 0x6D, 0x6E, 0x6E, 0x6F, 0x6F,
                    0x70, 0x70, 0x70, 0x71, 0x71, 0x72, 0x72, 0x73, 0x73, 0x73, 0x74, 0x74, 0x75, 0x75, 0x76, 0x76,
                    0x77, 0x77, 0x78, 0x78, 0x78, 0x79, 0x79, 0x7A, 0x7A, 0x7B, 0x7B, 0x7C, 0x7C, 0x7D, 0x7D, 0x7E,
                    0x7E, 0x7F, 0x7F, 0x80, 0x80, 0x81, 0x81, 0x82, 0x83, 0x83, 0x84, 0x84, 0x85, 0x85, 0x86, 0x86,
                    0x87, 0x88, 0x88, 0x89, 0x89, 0x8A, 0x8A, 0x8B, 0x8C, 0x8C, 0x8D, 0x8D, 0x8E, 0x8F, 0x8F, 0x90,
                    0x90, 0x91, 0x92, 0x92, 0x93, 0x94, 0x94, 0x95, 0x95, 0x96, 0x97, 0x97, 0x98, 0x99, 0x99, 0x9A,
                    0x9B, 0x9B, 0x9C, 0x9D, 0x9D, 0x9E, 0x9F, 0xA0, 0xA0, 0xA1, 0xA2, 0xA2, 0xA3, 0xA4, 0xA5, 0xA5,
                    0xA6, 0xA7, 0xA7, 0xA8, 0xA9, 0xAA, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAE, 0xAF, 0xB0, 0xB1, 0xB1,
                    0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
                    0xC0, 0xC1, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD,
                    0xCE, 0xCF, 0xD0, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD,
                    0xDE, 0xDF, 0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xED, 0xEE,
                    0xEF, 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFF, 0xFF
                ];
                let mut v = 0;
                for i in 0..0x80 {
                    let index: usize = v >> 13;
                    if index >= 0x28E {
                        // If we get above that value it means that the rest of the table is just all 0xFF
                        for insertron in i..0x80 {
                            self.ath_table[insertron] = 0xFF;
                        }
                        return Ok(());
                    }
                    self.ath_table[i] = list[index];
                    v += self.hca_header.sampling_rate as usize;
                }
                Ok(())
            },
            _ => { Err(GICSError::new("ATH table kind unknown. What kind of ATH table are you using, dear?")) }
        }
    }

    fn checksum(data: &[u8], size: usize) -> u16 {
        let v: [u16; 0x100] = [
            0x0000, 0x8005, 0x800F, 0x000A, 0x801B, 0x001E, 0x0014, 0x8011, 0x8033, 0x0036, 0x003C, 0x8039, 0x0028, 0x802D, 0x8027, 0x0022,
            0x8063, 0x0066, 0x006C, 0x8069, 0x0078, 0x807D, 0x8077, 0x0072, 0x0050, 0x8055, 0x805F, 0x005A, 0x804B, 0x004E, 0x0044, 0x8041,
            0x80C3, 0x00C6, 0x00CC, 0x80C9, 0x00D8, 0x80DD, 0x80D7, 0x00D2, 0x00F0, 0x80F5, 0x80FF, 0x00FA, 0x80EB, 0x00EE, 0x00E4, 0x80E1,
            0x00A0, 0x80A5, 0x80AF, 0x00AA, 0x80BB, 0x00BE, 0x00B4, 0x80B1, 0x8093, 0x0096, 0x009C, 0x8099, 0x0088, 0x808D, 0x8087, 0x0082,
            0x8183, 0x0186, 0x018C, 0x8189, 0x0198, 0x819D, 0x8197, 0x0192, 0x01B0, 0x81B5, 0x81BF, 0x01BA, 0x81AB, 0x01AE, 0x01A4, 0x81A1,
            0x01E0, 0x81E5, 0x81EF, 0x01EA, 0x81FB, 0x01FE, 0x01F4, 0x81F1, 0x81D3, 0x01D6, 0x01DC, 0x81D9, 0x01C8, 0x81CD, 0x81C7, 0x01C2,
            0x0140, 0x8145, 0x814F, 0x014A, 0x815B, 0x015E, 0x0154, 0x8151, 0x8173, 0x0176, 0x017C, 0x8179, 0x0168, 0x816D, 0x8167, 0x0162,
            0x8123, 0x0126, 0x012C, 0x8129, 0x0138, 0x813D, 0x8137, 0x0132, 0x0110, 0x8115, 0x811F, 0x011A, 0x810B, 0x010E, 0x0104, 0x8101,
            0x8303, 0x0306, 0x030C, 0x8309, 0x0318, 0x831D, 0x8317, 0x0312, 0x0330, 0x8335, 0x833F, 0x033A, 0x832B, 0x032E, 0x0324, 0x8321,
            0x0360, 0x8365, 0x836F, 0x036A, 0x837B, 0x037E, 0x0374, 0x8371, 0x8353, 0x0356, 0x035C, 0x8359, 0x0348, 0x834D, 0x8347, 0x0342,
            0x03C0, 0x83C5, 0x83CF, 0x03CA, 0x83DB, 0x03DE, 0x03D4, 0x83D1, 0x83F3, 0x03F6, 0x03FC, 0x83F9, 0x03E8, 0x83ED, 0x83E7, 0x03E2,
            0x83A3, 0x03A6, 0x03AC, 0x83A9, 0x03B8, 0x83BD, 0x83B7, 0x03B2, 0x0390, 0x8395, 0x839F, 0x039A, 0x838B, 0x038E, 0x0384, 0x8381,
            0x0280, 0x8285, 0x828F, 0x028A, 0x829B, 0x029E, 0x0294, 0x8291, 0x82B3, 0x02B6, 0x02BC, 0x82B9, 0x02A8, 0x82AD, 0x82A7, 0x02A2,
            0x82E3, 0x02E6, 0x02EC, 0x82E9, 0x02F8, 0x82FD, 0x82F7, 0x02F2, 0x02D0, 0x82D5, 0x82DF, 0x02DA, 0x82CB, 0x02CE, 0x02C4, 0x82C1,
            0x8243, 0x0246, 0x024C, 0x8249, 0x0258, 0x825D, 0x8257, 0x0252, 0x0270, 0x8275, 0x827F, 0x027A, 0x826B, 0x026E, 0x0264, 0x8261,
            0x0220, 0x8225, 0x822F, 0x022A, 0x823B, 0x023E, 0x0234, 0x8231, 0x8213, 0x0216, 0x021C, 0x8219, 0x0208, 0x820D, 0x8207, 0x0202
        ];

        let mut sum: u16 = 0;
        for i in 0..size {
            sum = sum << 8 ^ v[usize::from(sum >> 8 ^ u16::from(data[i]))];
        }
        sum
    }

    pub fn convert_to_wav(mut self, path: &Path) -> GICSResult<()> {
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
        wav_riff.fmt_sampling_per_sec = wav_riff.fmt_sampling_rate * u32::from(wav_riff.fmt_sampling_size);

        // Fill in wav sample
        let wav_simp = WaveSample::default();
        let mut wav_data = WaveData::default();
        wav_data.set_data_size(self.hca_header.block_count * 0x80 * 8 * u32::from(wav_riff.fmt_sampling_size) + (wav_simp.loop_end - wav_simp.loop_start) * loop_flag);
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
        wav_file.write_all(&header)?;

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
                            0x18 => (f * f64::from(0x0080_0000 - 1)).trunc(), // 24 bits
                            0x20 => (f * f64::from(i32::MAX)).trunc(), // 32 bits
                            _ => {
                                return Err(GICSError::new("Mode not supported: {}"));
                            }
                        } as i32;
                        /*if f != 0.0 {
                            println!("{} -> {}", f, v);
                        }*/
                        let max_bytes = usize::from(mode / 0x08);
                        wav_file.write_all(&v.to_le_bytes()[0..max_bytes])?;
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
            self.hca_channel[i as usize].decode_one(&mut data_block, self.hca_header.comp_r09, a, self.ath_table.as_ref());
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
            (0..channel_count-1).for_each(|j| {
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
        });
    }
}