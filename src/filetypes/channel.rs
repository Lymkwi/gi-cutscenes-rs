#[derive(Copy, Clone)]
pub struct Channel {
    block: [f32; 0x80],
    base_table: [f32; 0x80],
    value: [i8; 0x80],
    scale: [i8; 0x80],
    value2: [i8; 8],
    r#type: i32,
    value_3i: u32,
    count: u32,
    wav1: [f32; 0x80],
    wav2: [f32; 0x80],
    wav3: [f32; 0x80],
    wave: [[f32; 0x80]; 8]
}

const SCALE_LIST: [u8; 64] = [
    0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0D, 0x0D,
    0x0D, 0x0D, 0x0D, 0x0D, 0x0C, 0x0C, 0x0C, 0x0C,
    0x0C, 0x0C, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B, 0x0B,
    0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x09,
    0x09, 0x09, 0x09, 0x09, 0x09, 0x08, 0x08, 0x08,
    0x08, 0x08, 0x08, 0x07, 0x06, 0x06, 0x05, 0x04,
    0x04, 0x04, 0x03, 0x03, 0x03, 0x02, 0x02, 0x02,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];

const VALUE_INT: [u32; 64] = [
    0x342A_8D26, 0x3463_3F89, 0x3497_657D, 0x34C9_B9BE, 0x3506_6491, 0x3533_11C4, 0x356E_9910, 0x359E_F532,
    0x35D3_CCF1, 0x360D_1ADF, 0x363C_034A, 0x367A_83B3, 0x36A6_E595, 0x36DE_60F5, 0x3714_26FF, 0x3745_672A,
    0x3783_8359, 0x37AF_3B79, 0x37E9_7C38, 0x381B_8D3A, 0x384F_4319, 0x388A_14D5, 0x38B7_FBF0, 0x38F5_257D,
    0x3923_520F, 0x3959_9D16, 0x3990_FA4D, 0x39C1_2C4D, 0x3A00_B1ED, 0x3A2B_7A3A, 0x3A64_7B6D, 0x3A98_37F0,
    0x3ACA_D226, 0x3B07_1F62, 0x3B34_0AAF, 0x3B6F_E4BA, 0x3B9F_D228, 0x3BD4_F35B, 0x3C0D_DF04, 0x3C3D_08A4,
    0x3C7B_DFED, 0x3CA7_CD94, 0x3CDF_9613, 0x3D14_F4F0, 0x3D46_7991, 0x3D84_3A29, 0x3DB0_2F0E, 0x3DEA_C0C7,
    0x3E1C_6573, 0x3E50_6334, 0x3E8A_D4C6, 0x3EB8_FBAF, 0x3EF6_7A41, 0x3F24_3516, 0x3F5A_CB94, 0x3F91_C3D3,
    0x3FC2_38D2, 0x4001_64D2, 0x402C_6897, 0x4065_B907, 0x4099_0B88, 0x40CB_EC15, 0x4107_DB35, 0x4135_04F3
];

const SCALE_INT: [u32; 16] = [
    0x0000_0000, 0x3F2A_AAAB, 0x3ECC_CCCD, 0x3E92_4925, 0x3E63_8E39, 0x3E3A_2E8C, 0x3E1D_89D9, 0x3E08_8889,
    0x3D84_2108, 0x3D02_0821, 0x3C81_0204, 0x3C00_8081, 0x3B80_4020, 0x3B00_2008, 0x3A80_1002, 0x3A00_0801,
];

impl Channel {
    const fn new() -> Self {
        Self {
            block: [0.0; 0x80],
            base_table: [0.0_f32; 0x80],
            value: [0; 0x80],
            scale: [0; 0x80],
            value2: [0; 8],
            r#type: 0,
            value_3i: 0,
            count: 0,
            wav1: [0.0; 0x80],
            wav2: [0.0; 0x80],
            wav3: [0.0; 0x80],
            wave: [[0.0; 0x80]; 8]
        }
    }

    fn decode_one(&mut self, data: &mut ClData, a: u32, b: i32, ath_table: &[u8]) {
        let value_float_i: u32 = 0;
        let scale_float_i: u32 = 0;
        let mut v: i32 = data.get_bit(3);
        if v >= 6 {
            for i in 0..self.count {
                self.value[i as usize] = data.get_bit(6) as i8;
            }
        } else if v != 0 {
            let mut v1 = data.get_bit(6);
            let v2 = (1 << v) - 1;
            let v3 = v2 >> 1;
            let mut v4;
            self.value[0] = v1 as i8;
            for i in 1..self.count {
                v4 = data.get_bit(v);
                if v4 == v2 {
                    v1 = data.get_bit(6);
                } else {
                    v1 += v4 - v3;
                }
                self.value[i as usize] = v1 as i8;
            }
        } else {
            self.value = [0; 0x80];
        }

        if self.r#type == 2 {
            v = data.check_bit(4);
            self.value2[0] = v.try_into().unwrap();
            if v < 15 {
                for i in 0..self.value2.len() {
                    self.value2[i] = data.get_bit(4).try_into().unwrap();
                }
            }
        } else {
            for i in 0..a {
                self.value[(self.value_3i + i) as usize] = data.get_bit(6).try_into().unwrap();
            }
        }

        for i in 0..self.count {
            v = i32::from(self.value[i as usize]);
            if v != 0 {
                v = i32::from(ath_table[i as usize]) + ((b + (i as i32)) >> 8) - v * 5 / 2 + 1;
                if v < 0 {
                    v = 15;
                } else if v >= 0x39 {
                    v = 1;
                } else {
                    v = i32::from(SCALE_LIST[v as usize]);
                }
            }
            self.scale[i as usize] = v.try_into().unwrap();
        }

        for i in self.count..0x80 {
            self.scale[i as usize] = 0;
        }

        for i in 0..self.count {
            let mul: f32 = if self.value[i as usize] < 64 && self.value[i as usize] >= 0 {
                f32::from_bits(VALUE_INT[(value_float_i + self.value[i as usize] as u32) as usize])
            } else { 0.0 };
            let own_scale_idx = self.scale[i as usize] as u32;
            let def_scale_idx = scale_float_i + own_scale_idx;
            self.base_table[i as usize] = mul * f32::from_bits(
                SCALE_INT[def_scale_idx as usize]
            );
        }
    }

    fn decode_two(&mut self, data: &mut ClData) {
        let list1: [i8; 0x10] = [ 0, 2, 3, 3, 4, 4, 4, 4, 5, 6, 7, 8, 9, 10, 11, 12 ];
        let list2: [i8; 0x80] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 1, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            2, 2, 2, 2, 2, 2, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0,
            2, 2, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4,
            3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
            3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4
        ];
        let list3: [f32; 0x80] = [
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 1.0, -1.0, -1.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 1.0, -1.0, -1.0, 2.0, 2.0, -2.0, -2.0, 3.0, 3.0, -3.0, -3.0, 4.0, -4.0,
            0.0, 0.0, 1.0, 1.0, -1.0, -1.0, 2.0, 2.0, -2.0, -2.0, 3.0, -3.0, 4.0, -4.0, 5.0, -5.0,
            0.0, 0.0, 1.0, 1.0, -1.0, -1.0, 2.0, -2.0, 3.0, -3.0, 4.0, -4.0, 5.0, -5.0, 6.0, -6.0,
            0.0, 0.0, 1.0, -1.0, 2.0, -2.0, 3.0, -3.0, 4.0, -4.0, 5.0, -5.0, 6.0, -6.0, 7.0, -7.0
        ];

        (0..self.count).for_each(|i| {
            let index: usize = i as usize;

            let s: isize = self.scale[index].try_into().unwrap();
            let bit_size = list1[s as usize];
            let mut v = data.get_bit(i32::from(bit_size)) as isize;
            let f: f32 = if s < 8 {
                let shifted: isize = s << 4;
                v += shifted;
                data.add_bit(i32::from(list2[v as usize] - bit_size));
                list3[v as usize]
            } else {
                v = (1 - ((v & 1) << 1)) * (v / 2);
                if v == 0 { data.add_bit(-1); }
                v as f32
            };

            self.block[index] = self.base_table[index] * f;
        });

        // Clear the table between these two points
        let begin: usize = self.count as usize;
        let size: usize = 0x80 - begin;
        self.block[begin .. begin+size].iter_mut().for_each(|x| *x = 0.0);
    }

    fn decode_three(&mut self, param_alpha: u32, param_beta: u32, param_gamma: u32, param_delta: u32) {
        if self.r#type != 2 && param_beta > 0 {
            let list_float: [u32; 0x40] = [
                0x3F80_0000, 0x3FAA_8D26, 0x3FE3_3F89, 0x4017_657D, 0x4049_B9BE, 0x4086_6491, 0x40B3_11C4, 0x40EE_9910,
                0x411E_F532, 0x4153_CCF1, 0x418D_1ADF, 0x41BC_034A, 0x41FA_83B3, 0x4226_E595, 0x425E_60F5, 0x4294_26FF,
                0x42C5_672A, 0x4303_8359, 0x432F_3B79, 0x4369_7C38, 0x439B_8D3A, 0x43CF_4319, 0x440A_14D5, 0x4437_FBF0,
                0x4475_257D, 0x44A3_520F, 0x44D9_9D16, 0x4510_FA4D, 0x4541_2C4D, 0x4580_B1ED, 0x45AB_7A3A, 0x45E4_7B6D,
                0x4618_37F0, 0x464A_D226, 0x4687_1F62, 0x46B4_0AAF, 0x46EF_E4BA, 0x471F_D228, 0x4754_F35B, 0x478D_DF04,
                0x47BD_08A4, 0x47FB_DFED, 0x4827_CD94, 0x485F_9613, 0x4894_F4F0, 0x48C6_7991, 0x4904_3A29, 0x4930_2F0E,
                0x496A_C0C7, 0x499C_6573, 0x49D0_6334, 0x4A0A_D4C6, 0x4A38_FBAF, 0x4A76_7A41, 0x4AA4_3516, 0x4ADA_CB94,
                0x4B11_C3D3, 0x4B42_38D2, 0x4B81_64D2, 0x4BAC_6897, 0x4BE5_B907, 0x4C19_0B88, 0x4C4B_EC15, 0x0000_0000      
            ];
            for i in 0..param_alpha {
                let mut j = 0;
                let mut k = param_gamma;
                let mut l = param_gamma - 1;
                while j < param_beta && k < param_delta {
                    self.block[k as usize] = f32::from_bits(list_float[(self.value[(self.value_3i + i) as usize] - self.value[l as usize]) as usize]) * self.block[l as usize];
                    k += 1;
                    j += 1;
                    l -= 1;
                }
            }
            self.block[0x7F] = 0.0;
        }
    }

    fn decode_four(&mut self, index: usize, a: u32, b: u32, c: u32, chan: &Self) -> Self {
        let mut new_channel = *chan;
        if self.r#type != 1 && c != 0 {
            let list: [u32; 0x10] = [
                0x4000_0000, 0x3FED_B6DB, 0x3FDB_6DB7, 0x3FC9_2492, 0x3FB6_DB6E, 0x3FA4_9249, 0x3F92_4925, 0x3F80_0000,
                0x3F5B_6DB7, 0x3F36_DB6E, 0x3F12_4925, 0x3EDB_6DB7, 0x3E92_4925, 0x3E12_4925, 0x0000_0000, 0x0000_0000
            ];
            let f1 = f32::from_bits(list[new_channel.value2[index] as usize]);
            let f2 = f1 - 2.0;

            let mut self_index: usize = b as usize;
            let mut data_index: usize = b as usize;
            (0..a).for_each(|_| {
                new_channel.block[data_index] = self.block[self_index] * f2;
                self.block[self_index] *= f1;
                data_index += 1;
                self_index += 1;
            });
        }
        new_channel
    }

    #[allow(clippy::too_many_lines)]
    fn decode_five(&mut self, index: usize) {
        let substitutions: [[[u32; 0x40]; 7]; 2] = [
            // First block, aka list1Int
            [
                [
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75,
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75,
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75,
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75,
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75,
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75,
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75,
                    0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75, 0x3DA7_3D75
                ],
                [
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31,
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31,
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31,
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31,
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31,
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31,
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31,
                    0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31, 0x3F7B_14BE, 0x3F54_DB31
                ],
                [
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403,
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403,
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403,
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403,
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403,
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403,
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403,
                    0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403, 0x3F7E_C46D, 0x3F74_FA0B, 0x3F61_C598, 0x3F45_E403
                ],
                [
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9,
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9,
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9,
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9,
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9,
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9,
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9,
                    0x3F7F_B10F, 0x3F7D_3AAC, 0x3F78_53F8, 0x3F71_0908, 0x3F67_6BD8, 0x3F5B_941A, 0x3F4D_9F02, 0x3F3D_AEF9
                ],
                [
                    0x3F7F_EC43, 0x3F7F_4E6D, 0x3F7E_1324, 0x3F7C_3B28, 0x3F79_C79D, 0x3F76_BA07, 0x3F73_1447, 0x3F6E_D89E,
                    0x3F6A_09A7, 0x3F64_AA59, 0x3F5E_BE05, 0x3F58_4853, 0x3F51_4D3D, 0x3F49_D112, 0x3F41_D870, 0x3F39_6842,
                    0x3F7F_EC43, 0x3F7F_4E6D, 0x3F7E_1324, 0x3F7C_3B28, 0x3F79_C79D, 0x3F76_BA07, 0x3F73_1447, 0x3F6E_D89E,
                    0x3F6A_09A7, 0x3F64_AA59, 0x3F5E_BE05, 0x3F58_4853, 0x3F51_4D3D, 0x3F49_D112, 0x3F41_D870, 0x3F39_6842,
                    0x3F7F_EC43, 0x3F7F_4E6D, 0x3F7E_1324, 0x3F7C_3B28, 0x3F79_C79D, 0x3F76_BA07, 0x3F73_1447, 0x3F6E_D89E,
                    0x3F6A_09A7, 0x3F64_AA59, 0x3F5E_BE05, 0x3F58_4853, 0x3F51_4D3D, 0x3F49_D112, 0x3F41_D870, 0x3F39_6842,
                    0x3F7F_EC43, 0x3F7F_4E6D, 0x3F7E_1324, 0x3F7C_3B28, 0x3F79_C79D, 0x3F76_BA07, 0x3F73_1447, 0x3F6E_D89E,
                    0x3F6A_09A7, 0x3F64_AA59, 0x3F5E_BE05, 0x3F58_4853, 0x3F51_4D3D, 0x3F49_D112, 0x3F41_D870, 0x3F39_6842
                ],
                [
                    0x3F7F_FB11, 0x3F7F_D397, 0x3F7F_84AB, 0x3F7F_0E58, 0x3F7E_70B0, 0x3F7D_ABCC, 0x3F7C_BFC9, 0x3F7B_ACCD,
                    0x3F7A_7302, 0x3F79_1298, 0x3F77_8BC5, 0x3F75_DEC6, 0x3F74_0BDD, 0x3F72_1352, 0x3F6F_F573, 0x3F6D_B293,
                    0x3F6B_4B0C, 0x3F68_BF3C, 0x3F66_0F88, 0x3F63_3C5A, 0x3F60_4621, 0x3F5D_2D53, 0x3F59_F26A, 0x3F56_95E5,
                    0x3F53_1849, 0x3F4F_7A1F, 0x3F4B_BBF8, 0x3F47_DE65, 0x3F43_E200, 0x3F3F_C767, 0x3F3B_8F3B, 0x3F37_3A23,
                    0x3F7F_FB11, 0x3F7F_D397, 0x3F7F_84AB, 0x3F7F_0E58, 0x3F7E_70B0, 0x3F7D_ABCC, 0x3F7C_BFC9, 0x3F7B_ACCD,
                    0x3F7A_7302, 0x3F79_1298, 0x3F77_8BC5, 0x3F75_DEC6, 0x3F74_0BDD, 0x3F72_1352, 0x3F6F_F573, 0x3F6D_B293,
                    0x3F6B_4B0C, 0x3F68_BF3C, 0x3F66_0F88, 0x3F63_3C5A, 0x3F60_4621, 0x3F5D_2D53, 0x3F59_F26A, 0x3F56_95E5,
                    0x3F53_1849, 0x3F4F_7A1F, 0x3F4B_BBF8, 0x3F47_DE65, 0x3F43_E200, 0x3F3F_C767, 0x3F3B_8F3B, 0x3F37_3A23
                ],
                [
                    0x3F7F_FEC4, 0x3F7F_F4E6, 0x3F7F_E129, 0x3F7F_C38F, 0x3F7F_9C18, 0x3F7F_6AC7, 0x3F7F_2F9D, 0x3F7E_EA9D,
                    0x3F7E_9BC9, 0x3F7E_4323, 0x3F7D_E0B1, 0x3F7D_7474, 0x3F7C_FE73, 0x3F7C_7EB0, 0x3F7B_F531, 0x3F7B_61FC,
                    0x3F7A_C516, 0x3F7A_1E84, 0x3F79_6E4E, 0x3F78_B47B, 0x3F77_F110, 0x3F77_2417, 0x3F76_4D97, 0x3F75_6D97,
                    0x3F74_8422, 0x3F73_913F, 0x3F72_94F8, 0x3F71_8F57, 0x3F70_8066, 0x3F6F_6830, 0x3F6E_46BE, 0x3F6D_1C1D,
                    0x3F6B_E858, 0x3F6A_AB7B, 0x3F69_6591, 0x3F68_16A8, 0x3F66_BECC, 0x3F65_5E0B, 0x3F63_F473, 0x3F62_8210,
                    0x3F61_06F2, 0x3F5F_8327, 0x3F5D_F6BE, 0x3F5C_61C7, 0x3F5A_C450, 0x3F59_1E6A, 0x3F57_7026, 0x3F55_B993,
                    0x3F53_FAC3, 0x3F52_33C6, 0x3F50_64AF, 0x3F4E_8D90, 0x3F4C_AE79, 0x3F4A_C77F, 0x3F48_D8B3, 0x3F46_E22A,
                    0x3F44_E3F5, 0x3F42_DE29, 0x3F40_D0DA, 0x3F3E_BC1B, 0x3F3C_A003, 0x3F3A_7CA4, 0x3F38_5216, 0x3F36_206C
                ]
            ],
            [
                [
                    0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4,
                    0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4,
                    0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4,
                    0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4,
                    0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4,
                    0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4,
                    0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4,
                    0x3D0A_8BD4, 0xBD0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4, 0x3D0A_8BD4, 0x3D0A_8BD4, 0xBD0A_8BD4
                ],
                [
                    0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA,
                    0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA,
                    0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA,
                    0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA,
                    0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA,
                    0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA,
                    0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA,
                    0x3E47_C5C2, 0x3F0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0xBE47_C5C2, 0xBF0E_39DA, 0x3E47_C5C2, 0x3F0E_39DA
                ],
                [
                    0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799, 0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799,
                    0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799, 0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799,
                    0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799, 0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799,
                    0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799, 0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799,
                    0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799, 0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799,
                    0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799, 0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799,
                    0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799, 0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799,
                    0x3DC8_BD36, 0x3E94_A031, 0x3EF1_5AEA, 0x3F22_6799, 0xBDC8_BD36, 0xBE94_A031, 0xBEF1_5AEA, 0xBF22_6799
                ],
                [
                    0xBD48_FB30, 0xBE16_4083, 0xBE78_CFCC, 0xBEAC_7CD4, 0xBEDA_E880, 0xBF03_9C3D, 0xBF18_7FC0, 0xBF2B_EB4A,
                    0x3D48_FB30, 0x3E16_4083, 0x3E78_CFCC, 0x3EAC_7CD4, 0x3EDA_E880, 0x3F03_9C3D, 0x3F18_7FC0, 0x3F2B_EB4A,
                    0x3D48_FB30, 0x3E16_4083, 0x3E78_CFCC, 0x3EAC_7CD4, 0x3EDA_E880, 0x3F03_9C3D, 0x3F18_7FC0, 0x3F2B_EB4A,
                    0xBD48_FB30, 0xBE16_4083, 0xBE78_CFCC, 0xBEAC_7CD4, 0xBEDA_E880, 0xBF03_9C3D, 0xBF18_7FC0, 0xBF2B_EB4A,
                    0x3D48_FB30, 0x3E16_4083, 0x3E78_CFCC, 0x3EAC_7CD4, 0x3EDA_E880, 0x3F03_9C3D, 0x3F18_7FC0, 0x3F2B_EB4A,
                    0xBD48_FB30, 0xBE16_4083, 0xBE78_CFCC, 0xBEAC_7CD4, 0xBEDA_E880, 0xBF03_9C3D, 0xBF18_7FC0, 0xBF2B_EB4A,
                    0xBD48_FB30, 0xBE16_4083, 0xBE78_CFCC, 0xBEAC_7CD4, 0xBEDA_E880, 0xBF03_9C3D, 0xBF18_7FC0, 0xBF2B_EB4A,
                    0x3D48_FB30, 0x3E16_4083, 0x3E78_CFCC, 0x3EAC_7CD4, 0x3EDA_E880, 0x3F03_9C3D, 0x3F18_7FC0, 0x3F2B_EB4A
                ],
                [
                    0xBCC9_0AB0, 0xBD96_A905, 0xBDFA_B273, 0xBE2F_10A2, 0xBE60_5C13, 0xBE88_8E93, 0xBEA0_9AE5, 0xBEB8_442A,
                    0xBECF_7BCA, 0xBEE6_3375, 0xBEFC_5D27, 0xBF08_F59B, 0xBF13_682A, 0xBF1D_7FD1, 0xBF27_3656, 0xBF30_85BB,
                    0x3CC9_0AB0, 0x3D96_A905, 0x3DFA_B273, 0x3E2F_10A2, 0x3E60_5C13, 0x3E88_8E93, 0x3EA0_9AE5, 0x3EB8_442A,
                    0x3ECF_7BCA, 0x3EE6_3375, 0x3EFC_5D27, 0x3F08_F59B, 0x3F13_682A, 0x3F1D_7FD1, 0x3F27_3656, 0x3F30_85BB,
                    0x3CC9_0AB0, 0x3D96_A905, 0x3DFA_B273, 0x3E2F_10A2, 0x3E60_5C13, 0x3E88_8E93, 0x3EA0_9AE5, 0x3EB8_442A,
                    0x3ECF_7BCA, 0x3EE6_3375, 0x3EFC_5D27, 0x3F08_F59B, 0x3F13_682A, 0x3F1D_7FD1, 0x3F27_3656, 0x3F30_85BB,
                    0xBCC9_0AB0, 0xBD96_A905, 0xBDFA_B273, 0xBE2F_10A2, 0xBE60_5C13, 0xBE88_8E93, 0xBEA0_9AE5, 0xBEB8_442A,
                    0xBECF_7BCA, 0xBEE6_3375, 0xBEFC_5D27, 0xBF08_F59B, 0xBF13_682A, 0xBF1D_7FD1, 0xBF27_3656, 0xBF30_85BB
                ],
                [
                    0xBC49_0E90, 0xBD16_C32C, 0xBD7B_2B74, 0xBDAF_B680, 0xBDE1_BC2E, 0xBE09_CF86, 0xBE22_ABB6, 0xBE3B_6ECF,
                    0xBE54_1501, 0xBE6C_9A7F, 0xBE82_7DC0, 0xBE8E_9A22, 0xBE9A_A086, 0xBEA6_8F12, 0xBEB2_63EF, 0xBEBE_1D4A,
                    0xBEC9_B953, 0xBED5_3641, 0xBEE0_924F, 0xBEEB_CBBB, 0xBEF6_E0CB, 0xBF00_E7E4, 0xBF06_4B82, 0xBF0B_9A6B,
                    0xBF10_D3CD, 0xBF15_F6D9, 0xBF1B_02C6, 0xBF1F_F6CB, 0xBF24_D225, 0xBF29_9415, 0xBF2E_3BDE, 0xBF32_C8C9,
                    0x3C49_0E90, 0x3D16_C32C, 0x3D7B_2B74, 0x3DAF_B680, 0x3DE1_BC2E, 0x3E09_CF86, 0x3E22_ABB6, 0x3E3B_6ECF,
                    0x3E54_1501, 0x3E6C_9A7F, 0x3E82_7DC0, 0x3E8E_9A22, 0x3E9A_A086, 0x3EA6_8F12, 0x3EB2_63EF, 0x3EBE_1D4A,
                    0x3EC9_B953, 0x3ED5_3641, 0x3EE0_924F, 0x3EEB_CBBB, 0x3EF6_E0CB, 0x3F00_E7E4, 0x3F06_4B82, 0x3F0B_9A6B,
                    0x3F10_D3CD, 0x3F15_F6D9, 0x3F1B_02C6, 0x3F1F_F6CB, 0x3F24_D225, 0x3F29_9415, 0x3F2E_3BDE, 0x3F32_C8C9
                ],
                [
                    0xBBC9_0F88, 0xBC96_C9B6, 0xBCFB_49BA, 0xBD2F_E007, 0xBD62_1469, 0xBD8A_200A, 0xBDA3_308C, 0xBDBC_3AC3,
                    0xBDD5_3DB9, 0xBDEE_3876, 0xBE03_9502, 0xBE10_08B7, 0xBE1C_76DE, 0xBE28_DEFC, 0xBE35_4098, 0xBE41_9B37,
                    0xBE4D_EE60, 0xBE5A_3997, 0xBE66_7C66, 0xBE72_B651, 0xBE7E_E6E1, 0xBE85_86CE, 0xBE8B_9507, 0xBE91_9DDD,
                    0xBE97_A117, 0xBE9D_9E78, 0xBEA3_95C5, 0xBEA9_86C4, 0xBEAF_713A, 0xBEB5_54EC, 0xBEBB_31A0, 0xBEC1_071E,
                    0xBEC6_D529, 0xBECC_9B8B, 0xBED2_5A09, 0xBED8_106B, 0xBEDD_BE79, 0xBEE3_63FA, 0xBEE9_00B7, 0xBEEE_9479,
                    0xBEF4_1F07, 0xBEF9_A02D, 0xBEFF_17B2, 0xBF02_42B1, 0xBF04_F484, 0xBF07_A136, 0xBF0A_48AD, 0xBF0C_EAD0,
                    0xBF0F_8784, 0xBF12_1EB0, 0xBF14_B039, 0xBF17_3C07, 0xBF19_C200, 0xBF1C_420C, 0xBF1E_BC12, 0xBF21_2FF9,
                    0xBF23_9DA9, 0xBF26_050A, 0xBF28_6605, 0xBF2A_C082, 0xBF2D_1469, 0xBF2F_61A5, 0xBF31_A81D, 0xBF33_E7BC
                ]
            ]
        ];
        let subs_third: [[u32; 0x40]; 2] = [
            [
                0x3A35_04F0, 0x3B01_83B8, 0x3B70_C538, 0x3BBB_9268, 0x3C04_A809, 0x3C30_8200, 0x3C61_284C, 0x3C8B_3F17,
                0x3CA8_3992, 0x3CC7_7FBD, 0x3CE9_1110, 0x3D06_77CD, 0x3D19_8FC4, 0x3D2D_D35C, 0x3D43_4643, 0x3D59_ECC1,
                0x3D71_CBA8, 0x3D85_741E, 0x3D92_A413, 0x3DA0_78B4, 0x3DAE_F522, 0x3DBE_1C9E, 0x3DCD_F27B, 0x3DDE_7A1D,
                0x3DEF_B6ED, 0x3E00_D62B, 0x3E0A_2EDA, 0x3E13_E72A, 0x3E1E_00B1, 0x3E28_7CF2, 0x3E33_5D55, 0x3E3E_A321,
                0x3E4A_4F75, 0x3E56_633F, 0x3E62_DF37, 0x3E6F_C3D1, 0x3E7D_1138, 0x3E85_63A2, 0x3E8C_72B7, 0x3E93_B561,
                0x3E9B_2AEF, 0x3EA2_D26F, 0x3EAA_AAAB, 0x3EB2_B222, 0x3EBA_E706, 0x3EC3_4737, 0x3ECB_D03D, 0x3ED4_7F46,
                0x3EDD_5128, 0x3EE6_425C, 0x3EEF_4EFF, 0x3EF8_72D7, 0x3F00_D4A9, 0x3F05_76CA, 0x3F0A_1D3B, 0x3F0E_C548,
                0x3F13_6C25, 0x3F18_0EF2, 0x3F1C_AAC2, 0x3F21_3CA2, 0x3F25_C1A5, 0x3F2A_36E7, 0x3F2E_9998, 0x3F32_E705
            ],
            [
                0xBF37_1C9E, 0xBF3B_37FE, 0xBF3F_36F2, 0xBF43_1780, 0xBF46_D7E6, 0xBF4A_76A4, 0xBF4D_F27C, 0xBF51_4A6F,
                0xBF54_7DC5, 0xBF57_8C03, 0xBF5A_74EE, 0xBF5D_3887, 0xBF5F_D707, 0xBF62_50DA, 0xBF64_A699, 0xBF66_D908,
                0xBF68_E90E, 0xBF6A_D7B1, 0xBF6C_A611, 0xBF6E_5562, 0xBF6F_E6E7, 0xBF71_5BEF, 0xBF72_B5D1, 0xBF73_F5E6,
                0xBF75_1D89, 0xBF76_2E13, 0xBF77_28D7, 0xBF78_0F20, 0xBF78_E234, 0xBF79_A34C, 0xBF7A_5397, 0xBF7A_F439,
                0xBF7B_8648, 0xBF7C_0ACE, 0xBF7C_82C8, 0xBF7C_EF26, 0xBF7D_50CB, 0xBF7D_A88E, 0xBF7D_F737, 0xBF7E_3D86,
                0xBF7E_7C2A, 0xBF7E_B3CC, 0xBF7E_E507, 0xBF7F_106C, 0xBF7F_3683, 0xBF7F_57CA, 0xBF7F_74B6, 0xBF7F_8DB6,
                0xBF7F_A32E, 0xBF7F_B57B, 0xBF7F_C4F6, 0xBF7F_D1ED, 0xBF7F_DCAD, 0xBF7F_E579, 0xBF7F_EC90, 0xBF7F_F22E,
                0xBF7F_F688, 0xBF7F_F9D0, 0xBF7F_FC32, 0xBF7F_FDDA, 0xBF7F_FEED, 0xBF7F_FF8F, 0xBF7F_FFDF, 0xBF7F_FFFC
            ]
        ];

        let mut self_index = 0;
        let mut self_table: &mut [f32] = &mut self.block;
        let mut data_index = 0;
        let mut data_table: &mut [f32] = &mut self.wav1;

        let mut count_index = 0;
        let mut count1 = 1;
        let mut count2 = 0x40;

        while count_index < 7 {
            let mut d1 = data_index;
            let mut d2 = data_index + count2;
            (0..count1).for_each(|_| {
                (0..count2).for_each(|_| {
                    let a: f32 = self_table[self_index];
                    self_index += 1;
                    let b: f32 = self_table[self_index];
                    self_index += 1;

                    data_table[d1] = b + a;
                    data_table[d2] = a - b;
                    d1 += 1;
                    d2 += 1;
                });
                d1 += count2;
                d2 += count2;
            });
            let w = self_index - 0x80;
            let w_table: &mut [f32] = self_table;
            self_index = data_index;
            // Swap the tables;
            self_table = data_table;
            data_table = w_table;
            data_index = w;

            count_index += 1;
            count1 <<= 1;
            count2 >>= 1;
        }

        self_index = 0;
        self_table = &mut self.wav1;
        data_index = 0;
        data_table = &mut self.block;

        count_index = 0;
        count1 = 0x40;
        count2 = 1;
        while count_index < 7 {
            let mut list_1_float_i: usize = 0;
            let mut list_2_float_i: usize = 0;

            let mut p1 = self_index;
            let mut p2 = p1 + count2;
            let mut d1 = data_index;
            let mut d2 = d1 + (count2 * 2 - 1);

            (0..count1).for_each(|_| {
                (0..count2).for_each(|_| {
                    let a = self_table[p1]; p1 += 1;
                    let b = self_table[p2]; p2 += 1;

                    let c = f32::from_bits(substitutions[0][count_index][list_1_float_i]); list_1_float_i += 1;
                    let p = f32::from_bits(substitutions[1][count_index][list_2_float_i]); list_2_float_i += 1;

                    data_table[d1] = a * c - b * p; d1 += 1;
                    data_table[d2] = a.mul_add(p, b * c); d2 -= 1;
                });

                p1 += count2;
                p2 += count2;
                d1 += count2;
                d2 += count2 * 3;
            });

            // Swap the tables here as well
            let w = self_index;
            let w_table = self_table;
            self_index = data_index;
            self_table = data_table;
            data_index = w;
            data_table = w_table;

            count_index += 1;
            count1 >>= 1;
            count2 <<= 1;
        }

        data_index = 0;
        (0..0x80).for_each(|_| {
            self.wav2[data_index] = self_table[self_index];
            data_index += 1;
            self_index += 1;
        });
        self_index = 0;
        data_index = 0;
        let mut s1 = 0x40;
        let mut s2 = 0;

        (0..0x40).for_each(|_| {
            self.wave[index][data_index] = self.wav2[s1].mul_add(f32::from_bits(subs_third[0][self_index]), self.wav3[s2]);
            data_index += 1;
            self_index += 1;
            s1 += 1;
            s2 += 1;
        });

        self_index = 0;

        (0..0x40).for_each(|_| {
            s1 -= 1;
            self.wave[index][data_index] = f32::from_bits(subs_third[1][self_index]) * self.wav2[s1] - self.wav3[s2];
            data_index += 1;
            self_index += 1;
            s2 += 1;
        });

        self_index = 0x40;
        s1 = 0x40 - 1;
        s2 = 0;
        (0..0x40).for_each(|_| {
            self_index -= 1;
            self.wav3[s2] = self.wav2[s1] * f32::from_bits(subs_third[1][self_index]);
            s1 = s1.wrapping_sub(1);
            s2 += 1;
        });
        self_index = 0x40;
        (0..0x40).for_each(|_| {
            self_index -= 1;
            s1 = s1.wrapping_add(1);
            self.wav3[s2] = f32::from_bits(subs_third[0][self_index]) * self.wav2[s1];
            s2 += 1;
        });
    }
}

struct ClData {
    data: Vec<i32>,
    size: i32,
    bit: i32
}

const BIT_MASK: [i32; 8] = [ 0x00FF_FFFF, 0x007F_FFFF, 0x003F_FFFF, 0x001F_FFFF, 0x000F_FFFF, 0x0007_FFFF, 0x0003_FFFF, 0x0001_FFFF ];

impl ClData {
    fn new(data: Vec<i32>, size: i32) -> Self {
        Self {
            data,
            size: size * 8 - 16,
            bit: 0
        }
    }

    fn check_bit(&self, bit_size: i32) -> i32 {
        let mut v: i32 = 0;
        if self.bit + bit_size <= self.size {
            let data_offset = (self.bit >> 3) as usize;
            if data_offset >= self.data.len() {
                println!("{:?}", self.bit.checked_shr(3));
                return 0;
            }
            v = self.data[data_offset];
            v = v << 8 | (if data_offset + 1 < self.data.len() { self.data[data_offset + 1] } else { 0 });
            v = v << 8 | (if data_offset + 2 < self.data.len() { self.data[data_offset + 2] } else { 0 });
            v &= BIT_MASK[(self.bit & 7) as usize];
            v >>= 24 - (self.bit & 7) - bit_size;
        }
        v
    }

    fn get_bit(&mut self, bit_size: i32) -> i32 {
        let v = self.check_bit(bit_size);
        self.bit += bit_size;
        v
    }

    fn add_bit(&mut self, bit_size: i32) {
        self.bit += bit_size;
    }
}