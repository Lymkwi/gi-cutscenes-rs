
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
    filename: String,
    key1: [u8; 4],
    key2: [u8; 4],
    cipher_table: [u8; 0x100],
    ath_table: [u8; 0x100], // ?
    encrypted: bool,
    header: HCAHeader,
    hca_channel: Vec<Channel>
}

impl HCAFile {
    pub fn new(path: PathBuf, key1: [u8; 4], key2: [u8; 4]) -> Self {
        Self {
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            key1, key2,
            cipher_table: [0; 0x100],
            ath_table: [0; 0x100],
            encrypted: false,
            header: HCAHeader::default(),
            hca_channel: Vec::new()
        }
    }

    /*pub fn convert_to_wav(mut self, path: PathBuf) {
        // Some definitions
        let volume = 1;
        let mode = 16;
        let loop_flag = 0;

        let mut wav_riff = WaveRiff::new();
        wav_riff.fmt_type = if mode > 0 { 1 } else { 3 };
        wav_riff.fmt_channel_count = self.header.channel_count;
        wav_riff.fmt_bit_count = if mode > 0 { mode } else { 32 };
        wav_riff.fmt_sampling_rate = self.header.sampling_rate;
        wav_riff.fmt_sampling_size = wav_riff.fmt_bit_count / 8 * wav_riff.fmt_channel_count;

        // Fill in wav data
        //let wav_simp = WaveSimple::new();
        //let wav_data = WaveData::new();

    }*/
}