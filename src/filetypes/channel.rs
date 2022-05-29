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
    0x342A8D26, 0x34633F89, 0x3497657D, 0x34C9B9BE, 0x35066491, 0x353311C4, 0x356E9910, 0x359EF532,
    0x35D3CCF1, 0x360D1ADF, 0x363C034A, 0x367A83B3, 0x36A6E595, 0x36DE60F5, 0x371426FF, 0x3745672A,
    0x37838359, 0x37AF3B79, 0x37E97C38, 0x381B8D3A, 0x384F4319, 0x388A14D5, 0x38B7FBF0, 0x38F5257D,
    0x3923520F, 0x39599D16, 0x3990FA4D, 0x39C12C4D, 0x3A00B1ED, 0x3A2B7A3A, 0x3A647B6D, 0x3A9837F0,
    0x3ACAD226, 0x3B071F62, 0x3B340AAF, 0x3B6FE4BA, 0x3B9FD228, 0x3BD4F35B, 0x3C0DDF04, 0x3C3D08A4,
    0x3C7BDFED, 0x3CA7CD94, 0x3CDF9613, 0x3D14F4F0, 0x3D467991, 0x3D843A29, 0x3DB02F0E, 0x3DEAC0C7,
    0x3E1C6573, 0x3E506334, 0x3E8AD4C6, 0x3EB8FBAF, 0x3EF67A41, 0x3F243516, 0x3F5ACB94, 0x3F91C3D3,
    0x3FC238D2, 0x400164D2, 0x402C6897, 0x4065B907, 0x40990B88, 0x40CBEC15, 0x4107DB35, 0x413504F3
];

const SCALE_INT: [u32; 16] = [
    0x00000000, 0x3F2AAAAB, 0x3ECCCCCD, 0x3E924925, 0x3E638E39, 0x3E3A2E8C, 0x3E1D89D9, 0x3E088889,
    0x3D842108, 0x3D020821, 0x3C810204, 0x3C008081, 0x3B804020, 0x3B002008, 0x3A801002, 0x3A000801,
];

impl Channel {
    fn new() -> Self {
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

    fn decode_one(&mut self, data: &mut ClData, a: u32, b: i32, ath_table: Vec<u8>) {
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
                if v4 != v2 {
                    v1 += v4 - v3;
                } else {
                    v1 = data.get_bit(6);
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
            v = self.value[i as usize] as i32;
            if v != 0 {
                v = ath_table[i as usize] as i32 + (b + (i as i32) >> 8) - v * 5 / 2 + 1;
                if v < 0 {
                    v = 15;
                } else if v >= 0x39 {
                    v = 1;
                } else {
                    v = SCALE_LIST[v as usize] as i32;
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
            let mut v = data.get_bit(bit_size as i32) as isize;
            let f: f32 = if s < 0 {
                let shifted: isize = (s << 4).try_into().unwrap();
                v += shifted;
                data.add_bit((list2[v as usize] - bit_size) as i32);
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

    fn decode_three(&mut self, a: u32, b: u32, c: u32, d: u32) {
        if self.r#type != 2 && b > 0 {
            let list_float: [u32; 0x40] = [
                0x3F800000, 0x3FAA8D26, 0x3FE33F89, 0x4017657D, 0x4049B9BE, 0x40866491, 0x40B311C4, 0x40EE9910,
                0x411EF532, 0x4153CCF1, 0x418D1ADF, 0x41BC034A, 0x41FA83B3, 0x4226E595, 0x425E60F5, 0x429426FF,
                0x42C5672A, 0x43038359, 0x432F3B79, 0x43697C38, 0x439B8D3A, 0x43CF4319, 0x440A14D5, 0x4437FBF0,
                0x4475257D, 0x44A3520F, 0x44D99D16, 0x4510FA4D, 0x45412C4D, 0x4580B1ED, 0x45AB7A3A, 0x45E47B6D,
                0x461837F0, 0x464AD226, 0x46871F62, 0x46B40AAF, 0x46EFE4BA, 0x471FD228, 0x4754F35B, 0x478DDF04,
                0x47BD08A4, 0x47FBDFED, 0x4827CD94, 0x485F9613, 0x4894F4F0, 0x48C67991, 0x49043A29, 0x49302F0E,
                0x496AC0C7, 0x499C6573, 0x49D06334, 0x4A0AD4C6, 0x4A38FBAF, 0x4A767A41, 0x4AA43516, 0x4ADACB94,
                0x4B11C3D3, 0x4B4238D2, 0x4B8164D2, 0x4BAC6897, 0x4BE5B907, 0x4C190B88, 0x4C4BEC15, 0x00000000      
            ];
            for i in 0..a {
                let mut j = 0;
                let mut k = c;
                let mut l = c - 1;
                while j < b && k < d {
                    self.block[k as usize] = f32::from_bits(list_float[(self.value[(self.value_3i + i) as usize] - self.value[l as usize]) as usize]) * self.block[l as usize];
                    k += 1;
                    j += 1;
                    l -= 1;
                }
            }
            self.block[0x7F] = 0.0;
        }
    }

    fn decode_four(&mut self, index: usize, a: u32, b: u32, c: u32, chan: &Channel) -> Channel {
        let mut new_channel = chan.clone();
        if self.r#type != 1 && c != 0 {
            let list: [u32; 0x10] = [
                0x40000000, 0x3FEDB6DB, 0x3FDB6DB7, 0x3FC92492, 0x3FB6DB6E, 0x3FA49249, 0x3F924925, 0x3F800000,
                0x3F5B6DB7, 0x3F36DB6E, 0x3F124925, 0x3EDB6DB7, 0x3E924925, 0x3E124925, 0x00000000, 0x00000000
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
            })
        }
        new_channel
    }

    fn decode_five(&mut self, index: usize) {
        let substitutions: [[[u32; 0x40]; 7]; 2] = [
            // First block, aka list1Int
            [
                [
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75,
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75,
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75,
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75,
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75,
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75,
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75,
                    0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75, 0x3DA73D75
                ],
                [
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31,
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31,
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31,
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31,
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31,
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31,
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31,
                    0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31, 0x3F7B14BE, 0x3F54DB31
                ],
                [
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403,
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403,
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403,
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403,
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403,
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403,
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403,
                    0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403, 0x3F7EC46D, 0x3F74FA0B, 0x3F61C598, 0x3F45E403
                ],
                [
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9,
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9,
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9,
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9,
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9,
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9,
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9,
                    0x3F7FB10F, 0x3F7D3AAC, 0x3F7853F8, 0x3F710908, 0x3F676BD8, 0x3F5B941A, 0x3F4D9F02, 0x3F3DAEF9
                ],
                [
                    0x3F7FEC43, 0x3F7F4E6D, 0x3F7E1324, 0x3F7C3B28, 0x3F79C79D, 0x3F76BA07, 0x3F731447, 0x3F6ED89E,
                    0x3F6A09A7, 0x3F64AA59, 0x3F5EBE05, 0x3F584853, 0x3F514D3D, 0x3F49D112, 0x3F41D870, 0x3F396842,
                    0x3F7FEC43, 0x3F7F4E6D, 0x3F7E1324, 0x3F7C3B28, 0x3F79C79D, 0x3F76BA07, 0x3F731447, 0x3F6ED89E,
                    0x3F6A09A7, 0x3F64AA59, 0x3F5EBE05, 0x3F584853, 0x3F514D3D, 0x3F49D112, 0x3F41D870, 0x3F396842,
                    0x3F7FEC43, 0x3F7F4E6D, 0x3F7E1324, 0x3F7C3B28, 0x3F79C79D, 0x3F76BA07, 0x3F731447, 0x3F6ED89E,
                    0x3F6A09A7, 0x3F64AA59, 0x3F5EBE05, 0x3F584853, 0x3F514D3D, 0x3F49D112, 0x3F41D870, 0x3F396842,
                    0x3F7FEC43, 0x3F7F4E6D, 0x3F7E1324, 0x3F7C3B28, 0x3F79C79D, 0x3F76BA07, 0x3F731447, 0x3F6ED89E,
                    0x3F6A09A7, 0x3F64AA59, 0x3F5EBE05, 0x3F584853, 0x3F514D3D, 0x3F49D112, 0x3F41D870, 0x3F396842
                ],
                [
                    0x3F7FFB11, 0x3F7FD397, 0x3F7F84AB, 0x3F7F0E58, 0x3F7E70B0, 0x3F7DABCC, 0x3F7CBFC9, 0x3F7BACCD,
                    0x3F7A7302, 0x3F791298, 0x3F778BC5, 0x3F75DEC6, 0x3F740BDD, 0x3F721352, 0x3F6FF573, 0x3F6DB293,
                    0x3F6B4B0C, 0x3F68BF3C, 0x3F660F88, 0x3F633C5A, 0x3F604621, 0x3F5D2D53, 0x3F59F26A, 0x3F5695E5,
                    0x3F531849, 0x3F4F7A1F, 0x3F4BBBF8, 0x3F47DE65, 0x3F43E200, 0x3F3FC767, 0x3F3B8F3B, 0x3F373A23,
                    0x3F7FFB11, 0x3F7FD397, 0x3F7F84AB, 0x3F7F0E58, 0x3F7E70B0, 0x3F7DABCC, 0x3F7CBFC9, 0x3F7BACCD,
                    0x3F7A7302, 0x3F791298, 0x3F778BC5, 0x3F75DEC6, 0x3F740BDD, 0x3F721352, 0x3F6FF573, 0x3F6DB293,
                    0x3F6B4B0C, 0x3F68BF3C, 0x3F660F88, 0x3F633C5A, 0x3F604621, 0x3F5D2D53, 0x3F59F26A, 0x3F5695E5,
                    0x3F531849, 0x3F4F7A1F, 0x3F4BBBF8, 0x3F47DE65, 0x3F43E200, 0x3F3FC767, 0x3F3B8F3B, 0x3F373A23
                ],
                [
                    0x3F7FFEC4, 0x3F7FF4E6, 0x3F7FE129, 0x3F7FC38F, 0x3F7F9C18, 0x3F7F6AC7, 0x3F7F2F9D, 0x3F7EEA9D,
                    0x3F7E9BC9, 0x3F7E4323, 0x3F7DE0B1, 0x3F7D7474, 0x3F7CFE73, 0x3F7C7EB0, 0x3F7BF531, 0x3F7B61FC,
                    0x3F7AC516, 0x3F7A1E84, 0x3F796E4E, 0x3F78B47B, 0x3F77F110, 0x3F772417, 0x3F764D97, 0x3F756D97,
                    0x3F748422, 0x3F73913F, 0x3F7294F8, 0x3F718F57, 0x3F708066, 0x3F6F6830, 0x3F6E46BE, 0x3F6D1C1D,
                    0x3F6BE858, 0x3F6AAB7B, 0x3F696591, 0x3F6816A8, 0x3F66BECC, 0x3F655E0B, 0x3F63F473, 0x3F628210,
                    0x3F6106F2, 0x3F5F8327, 0x3F5DF6BE, 0x3F5C61C7, 0x3F5AC450, 0x3F591E6A, 0x3F577026, 0x3F55B993,
                    0x3F53FAC3, 0x3F5233C6, 0x3F5064AF, 0x3F4E8D90, 0x3F4CAE79, 0x3F4AC77F, 0x3F48D8B3, 0x3F46E22A,
                    0x3F44E3F5, 0x3F42DE29, 0x3F40D0DA, 0x3F3EBC1B, 0x3F3CA003, 0x3F3A7CA4, 0x3F385216, 0x3F36206C
                ]
            ],
            [
                [
                    0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4,
                    0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4,
                    0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4,
                    0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4,
                    0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4,
                    0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4,
                    0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4,
                    0x3D0A8BD4, 0xBD0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4, 0x3D0A8BD4, 0x3D0A8BD4, 0xBD0A8BD4
                ],
                [
                    0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA,
                    0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA,
                    0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA,
                    0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA,
                    0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA,
                    0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA,
                    0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA,
                    0x3E47C5C2, 0x3F0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0xBE47C5C2, 0xBF0E39DA, 0x3E47C5C2, 0x3F0E39DA
                ],
                [
                    0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799, 0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799,
                    0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799, 0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799,
                    0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799, 0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799,
                    0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799, 0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799,
                    0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799, 0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799,
                    0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799, 0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799,
                    0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799, 0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799,
                    0x3DC8BD36, 0x3E94A031, 0x3EF15AEA, 0x3F226799, 0xBDC8BD36, 0xBE94A031, 0xBEF15AEA, 0xBF226799
                ],
                [
                    0xBD48FB30, 0xBE164083, 0xBE78CFCC, 0xBEAC7CD4, 0xBEDAE880, 0xBF039C3D, 0xBF187FC0, 0xBF2BEB4A,
                    0x3D48FB30, 0x3E164083, 0x3E78CFCC, 0x3EAC7CD4, 0x3EDAE880, 0x3F039C3D, 0x3F187FC0, 0x3F2BEB4A,
                    0x3D48FB30, 0x3E164083, 0x3E78CFCC, 0x3EAC7CD4, 0x3EDAE880, 0x3F039C3D, 0x3F187FC0, 0x3F2BEB4A,
                    0xBD48FB30, 0xBE164083, 0xBE78CFCC, 0xBEAC7CD4, 0xBEDAE880, 0xBF039C3D, 0xBF187FC0, 0xBF2BEB4A,
                    0x3D48FB30, 0x3E164083, 0x3E78CFCC, 0x3EAC7CD4, 0x3EDAE880, 0x3F039C3D, 0x3F187FC0, 0x3F2BEB4A,
                    0xBD48FB30, 0xBE164083, 0xBE78CFCC, 0xBEAC7CD4, 0xBEDAE880, 0xBF039C3D, 0xBF187FC0, 0xBF2BEB4A,
                    0xBD48FB30, 0xBE164083, 0xBE78CFCC, 0xBEAC7CD4, 0xBEDAE880, 0xBF039C3D, 0xBF187FC0, 0xBF2BEB4A,
                    0x3D48FB30, 0x3E164083, 0x3E78CFCC, 0x3EAC7CD4, 0x3EDAE880, 0x3F039C3D, 0x3F187FC0, 0x3F2BEB4A
                ],
                [
                    0xBCC90AB0, 0xBD96A905, 0xBDFAB273, 0xBE2F10A2, 0xBE605C13, 0xBE888E93, 0xBEA09AE5, 0xBEB8442A,
                    0xBECF7BCA, 0xBEE63375, 0xBEFC5D27, 0xBF08F59B, 0xBF13682A, 0xBF1D7FD1, 0xBF273656, 0xBF3085BB,
                    0x3CC90AB0, 0x3D96A905, 0x3DFAB273, 0x3E2F10A2, 0x3E605C13, 0x3E888E93, 0x3EA09AE5, 0x3EB8442A,
                    0x3ECF7BCA, 0x3EE63375, 0x3EFC5D27, 0x3F08F59B, 0x3F13682A, 0x3F1D7FD1, 0x3F273656, 0x3F3085BB,
                    0x3CC90AB0, 0x3D96A905, 0x3DFAB273, 0x3E2F10A2, 0x3E605C13, 0x3E888E93, 0x3EA09AE5, 0x3EB8442A,
                    0x3ECF7BCA, 0x3EE63375, 0x3EFC5D27, 0x3F08F59B, 0x3F13682A, 0x3F1D7FD1, 0x3F273656, 0x3F3085BB,
                    0xBCC90AB0, 0xBD96A905, 0xBDFAB273, 0xBE2F10A2, 0xBE605C13, 0xBE888E93, 0xBEA09AE5, 0xBEB8442A,
                    0xBECF7BCA, 0xBEE63375, 0xBEFC5D27, 0xBF08F59B, 0xBF13682A, 0xBF1D7FD1, 0xBF273656, 0xBF3085BB
                ],
                [
                    0xBC490E90, 0xBD16C32C, 0xBD7B2B74, 0xBDAFB680, 0xBDE1BC2E, 0xBE09CF86, 0xBE22ABB6, 0xBE3B6ECF,
                    0xBE541501, 0xBE6C9A7F, 0xBE827DC0, 0xBE8E9A22, 0xBE9AA086, 0xBEA68F12, 0xBEB263EF, 0xBEBE1D4A,
                    0xBEC9B953, 0xBED53641, 0xBEE0924F, 0xBEEBCBBB, 0xBEF6E0CB, 0xBF00E7E4, 0xBF064B82, 0xBF0B9A6B,
                    0xBF10D3CD, 0xBF15F6D9, 0xBF1B02C6, 0xBF1FF6CB, 0xBF24D225, 0xBF299415, 0xBF2E3BDE, 0xBF32C8C9,
                    0x3C490E90, 0x3D16C32C, 0x3D7B2B74, 0x3DAFB680, 0x3DE1BC2E, 0x3E09CF86, 0x3E22ABB6, 0x3E3B6ECF,
                    0x3E541501, 0x3E6C9A7F, 0x3E827DC0, 0x3E8E9A22, 0x3E9AA086, 0x3EA68F12, 0x3EB263EF, 0x3EBE1D4A,
                    0x3EC9B953, 0x3ED53641, 0x3EE0924F, 0x3EEBCBBB, 0x3EF6E0CB, 0x3F00E7E4, 0x3F064B82, 0x3F0B9A6B,
                    0x3F10D3CD, 0x3F15F6D9, 0x3F1B02C6, 0x3F1FF6CB, 0x3F24D225, 0x3F299415, 0x3F2E3BDE, 0x3F32C8C9
                ],
                [
                    0xBBC90F88, 0xBC96C9B6, 0xBCFB49BA, 0xBD2FE007, 0xBD621469, 0xBD8A200A, 0xBDA3308C, 0xBDBC3AC3,
                    0xBDD53DB9, 0xBDEE3876, 0xBE039502, 0xBE1008B7, 0xBE1C76DE, 0xBE28DEFC, 0xBE354098, 0xBE419B37,
                    0xBE4DEE60, 0xBE5A3997, 0xBE667C66, 0xBE72B651, 0xBE7EE6E1, 0xBE8586CE, 0xBE8B9507, 0xBE919DDD,
                    0xBE97A117, 0xBE9D9E78, 0xBEA395C5, 0xBEA986C4, 0xBEAF713A, 0xBEB554EC, 0xBEBB31A0, 0xBEC1071E,
                    0xBEC6D529, 0xBECC9B8B, 0xBED25A09, 0xBED8106B, 0xBEDDBE79, 0xBEE363FA, 0xBEE900B7, 0xBEEE9479,
                    0xBEF41F07, 0xBEF9A02D, 0xBEFF17B2, 0xBF0242B1, 0xBF04F484, 0xBF07A136, 0xBF0A48AD, 0xBF0CEAD0,
                    0xBF0F8784, 0xBF121EB0, 0xBF14B039, 0xBF173C07, 0xBF19C200, 0xBF1C420C, 0xBF1EBC12, 0xBF212FF9,
                    0xBF239DA9, 0xBF26050A, 0xBF286605, 0xBF2AC082, 0xBF2D1469, 0xBF2F61A5, 0xBF31A81D, 0xBF33E7BC
                ]
            ]
        ];
        let subs_third: [[u32; 0x40]; 2] = [
            [
                0x3A3504F0, 0x3B0183B8, 0x3B70C538, 0x3BBB9268, 0x3C04A809, 0x3C308200, 0x3C61284C, 0x3C8B3F17,
                0x3CA83992, 0x3CC77FBD, 0x3CE91110, 0x3D0677CD, 0x3D198FC4, 0x3D2DD35C, 0x3D434643, 0x3D59ECC1,
                0x3D71CBA8, 0x3D85741E, 0x3D92A413, 0x3DA078B4, 0x3DAEF522, 0x3DBE1C9E, 0x3DCDF27B, 0x3DDE7A1D,
                0x3DEFB6ED, 0x3E00D62B, 0x3E0A2EDA, 0x3E13E72A, 0x3E1E00B1, 0x3E287CF2, 0x3E335D55, 0x3E3EA321,
                0x3E4A4F75, 0x3E56633F, 0x3E62DF37, 0x3E6FC3D1, 0x3E7D1138, 0x3E8563A2, 0x3E8C72B7, 0x3E93B561,
                0x3E9B2AEF, 0x3EA2D26F, 0x3EAAAAAB, 0x3EB2B222, 0x3EBAE706, 0x3EC34737, 0x3ECBD03D, 0x3ED47F46,
                0x3EDD5128, 0x3EE6425C, 0x3EEF4EFF, 0x3EF872D7, 0x3F00D4A9, 0x3F0576CA, 0x3F0A1D3B, 0x3F0EC548,
                0x3F136C25, 0x3F180EF2, 0x3F1CAAC2, 0x3F213CA2, 0x3F25C1A5, 0x3F2A36E7, 0x3F2E9998, 0x3F32E705
            ],
            [
                0xBF371C9E, 0xBF3B37FE, 0xBF3F36F2, 0xBF431780, 0xBF46D7E6, 0xBF4A76A4, 0xBF4DF27C, 0xBF514A6F,
                0xBF547DC5, 0xBF578C03, 0xBF5A74EE, 0xBF5D3887, 0xBF5FD707, 0xBF6250DA, 0xBF64A699, 0xBF66D908,
                0xBF68E90E, 0xBF6AD7B1, 0xBF6CA611, 0xBF6E5562, 0xBF6FE6E7, 0xBF715BEF, 0xBF72B5D1, 0xBF73F5E6,
                0xBF751D89, 0xBF762E13, 0xBF7728D7, 0xBF780F20, 0xBF78E234, 0xBF79A34C, 0xBF7A5397, 0xBF7AF439,
                0xBF7B8648, 0xBF7C0ACE, 0xBF7C82C8, 0xBF7CEF26, 0xBF7D50CB, 0xBF7DA88E, 0xBF7DF737, 0xBF7E3D86,
                0xBF7E7C2A, 0xBF7EB3CC, 0xBF7EE507, 0xBF7F106C, 0xBF7F3683, 0xBF7F57CA, 0xBF7F74B6, 0xBF7F8DB6,
                0xBF7FA32E, 0xBF7FB57B, 0xBF7FC4F6, 0xBF7FD1ED, 0xBF7FDCAD, 0xBF7FE579, 0xBF7FEC90, 0xBF7FF22E,
                0xBF7FF688, 0xBF7FF9D0, 0xBF7FFC32, 0xBF7FFDDA, 0xBF7FFEED, 0xBF7FFF8F, 0xBF7FFFDF, 0xBF7FFFFC
            ]
        ];

        let mut s = 0;
        let mut self_table: &mut [f32] = &mut self.block;
        let mut d = 0;
        let mut data_table: &mut [f32] = &mut self.wav1;

        let mut i = 0;
        let mut count1 = 1;
        let mut count2 = 0x40;

        while i < 7 {
            let mut d1 = d;
            let mut d2 = d + count2;
            (0..count1).for_each(|_| {
                (0..count2).for_each(|_| {
                    let a: f32 = self_table[s];
                    s += 1;
                    let b: f32 = self_table[s];
                    s += 1;

                    data_table[d1] = b + a;
                    data_table[d2] = a - b;
                });
                d1 += 1;
                d2 += 1;
            });
            let w = s - 0x80;
            let w_table: &mut [f32] = self_table;
            s = d;
            // Swap the tables;
            self_table = data_table;
            data_table = w_table;
            d = w;

            i += 1;
            count1 *= 2;
            count2 >>= 1;
        }

        s = 0;
        self_table = &mut self.wav1;
        d = 0;
        data_table = &mut self.block;

        i = 0;
        count1 = 0x40;
        count2 = 1;
        while i < 7 {
            let mut list_1_float_i: usize = 0;
            let mut list_2_float_i: usize = 0;

            let mut p1 = s;
            let mut p2 = p1 + count2;
            let mut d1 = d;
            let mut d2 = d1 + (count2 * 2 - 1);

            (0..count1).for_each(|_| {
                (0..count2).for_each(|_| {
                    let a = self_table[p1]; p1 += 1;
                    let b = self_table[p2]; p2 += 1;

                    let c = f32::from_bits(substitutions[0][i][list_1_float_i]); list_1_float_i += 1;
                    let p = f32::from_bits(substitutions[1][i][list_2_float_i]); list_2_float_i += 1;

                    data_table[d1] = a * c - b * p; d1 += 1;
                    data_table[d2] = a * p + b * c; d2 -= 1;
                });

                p1 += count2;
                p2 += count2;
                d1 += count2;
                d2 += count2 * 3;
            });

            // Swap the tables here as well
            let w = s;
            let w_table = self_table;
            s = d;
            self_table = data_table;
            d = w;
            data_table = w_table;

            i += 1;
            count1 >>= 1;
            count2 *= 2;
        }

        d = 0;
        (0..0x80).for_each(|_| {
            self.wav2[d] = self_table[s];
            d += 1;
            s += 1;
        });
        s = 0;
        d = 0;
        let mut s1 = 0x40;
        let mut s2 = 0;

        (0..0x40).for_each(|_| {
            self.wave[index][d] = self.wav2[s1] * f32::from_bits(subs_third[0][s]) + self.wav3[s2];
            d += 1;
            s += 1;
            s1 += 1;
            s2 += 1;
        });

        s = 0;

        (0..0x40).for_each(|_| {
            s1 -= 1;
            self.wave[index][d] = f32::from_bits(subs_third[1][s]) * self.wav2[s1] - self.wav3[s2];
            d += 1;
            s += 1;
            s2 += 1;
        });

        s = 0x40;
        s1 = 0x40 - 1;
        s2 = 0;
        (0..0x40).for_each(|_| {
            s -= 1;
            self.wav3[s2] = self.wav2[s1] * f32::from_bits(subs_third[1][s]);
            s1 = s1.wrapping_sub(1);
            s2 += 1;
        });
        s = 0x40;
        (0..0x40).for_each(|_| {
            s -= 1;
            s1 = s1.wrapping_add(1);
            self.wav3[s2] = f32::from_bits(subs_third[0][s]) * self.wav2[s1];
            s2 += 1;
        });
    }
}

struct ClData {
    data: Vec<i32>,
    size: i32,
    bit: i32
}

const BIT_MASK: [i32; 8] = [ 0xFFFFFF, 0x7FFFFF, 0x3FFFFF, 0x1FFFFF, 0x0FFFFF, 0x07FFFF, 0x03FFFF, 0x01FFFF ];

impl ClData {
    fn new(data: Vec<i32>, size: i32) -> Self {
        Self {
            data,
            size: size * 8 - 16,
            bit: 0
        }
    }

    fn get_data(&self) -> &[i32] {
        &self.data
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
        return v;
    }

    fn get_bit(&mut self, bit_size: i32) -> i32 {
        let v = self.check_bit(bit_size);
        self.bit += bit_size;
        return v;
    }

    fn add_bit(&mut self, bit_size: i32) {
        self.bit += bit_size;
    }
}