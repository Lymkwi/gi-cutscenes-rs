struct WaveRiff {
    riff: [u8; 4],
    riff_size: u32,

    wave: [u8; 4],

    fmt: [u8; 4],
    fmt_size: u32,
    fmt_type: u16,
    fmt_channel_count: u16,

    fmt_sampling_rate: u32,
    fmt_sampling_per_sec: u32,
    fmt_sampling_size: u16,
    fmt_bit_count: u16
}

impl Default for WaveRiff {
    fn default() -> Self {
        Self {
            riff: [82, 73, 70, 70], // "RIFF"
            riff_size: 0,
            wave: [87, 65, 86, 69], // "WAVE"
            fmt: [102, 109, 116, 32], // "fmt "
            fmt_size: 0x10,
            fmt_type: 0,
            fmt_channel_count: 0,
            fmt_sampling_rate: 0,
            fmt_sampling_per_sec: 0,
            fmt_sampling_size: 0,
            fmt_bit_count: 0
        }
    }
}

impl WaveRiff {
    fn build_byte_array(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.riff);
        result.extend(self.riff_size.to_le_bytes());
        result.extend(self.wave);
        result.extend(self.fmt);
        result.extend(self.fmt_size.to_le_bytes());
        result.extend(self.fmt_type.to_le_bytes());
        result.extend(self.fmt_channel_count.to_le_bytes());
        result.extend(self.fmt_sampling_rate.to_le_bytes());
        result.extend(self.fmt_sampling_per_sec.to_le_bytes());
        result.extend(self.fmt_sampling_size.to_le_bytes());
        result.extend(self.fmt_bit_count.to_le_bytes());

        result
    }
}

struct WaveSample {
    smpl: [u8; 4],
    smpl_size: u32,
    manufacturer: u32,
    product: u32,
    sample_period: u32,
    midi_unity_note: u32,
    midi_pitch_fraction: u32,
    smpte_format: u32,
    smpte_offset: u32,
    sample_loops: u32,
    sampler_data: u32,
    loop_identifier: u32,
    loop_type: u32,
    loop_start: u32,
    loop_end: u32,
    loop_fraction: u32,
    loop_play_count: u32
}

impl Default for WaveSample {
    fn default() -> Self {
        Self {
            smpl: [115, 109, 112, 108],
            smpl_size: 0x3C,
            manufacturer: 0,
            product: 0,
            sample_period: 0,
            midi_unity_note: 0,
            midi_pitch_fraction: 0,
            smpte_format: 0,
            smpte_offset: 0,
            sample_loops: 1,
            sampler_data: 0x18,
            loop_identifier: 0,
            loop_type: 0,
            loop_start: 0,
            loop_end: 0,
            loop_fraction: 0,
            loop_play_count: 0
        }
    }
}

pub struct WaveData {
    data: [u8; 4],
    data_size: u32
}

impl Default for WaveData {
    fn default() -> Self {
        Self {
            data: [100, 97, 116, 97],
            data_size: 0
        }
    }
}

impl WaveData {
    fn build_byte_array(&self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend(self.data);
        res.extend(self.data_size.to_le_bytes());

        return res;
    }
}