pub const fn make_be32(a: &[u8]) -> u32 {
    ((a[0] as u32) << 24)
    + ((a[1] as u32) << 16)
    + ((a[2] as u32) << 8)
    + (a[3] as u32)
}

pub fn make_be16(a: &[u8]) -> u16 {
    (u16::from(a[0]) << 8)
    + u16::from(a[1])
}