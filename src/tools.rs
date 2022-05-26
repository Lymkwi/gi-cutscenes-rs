pub fn bswap_u32(a: u32) -> u32 {
    a >> 24
    + (((a >> 16) & 0xFF) << 8)
    + (((a >>  8) & 0xFF) << 16)
    + ((a & 0xFF) << 24)
}

pub fn bswap_u16(a: u16) -> u16 {
    a >> 8 + ((a & 0xFF) << 8)
}

pub fn make_be32(a: &[u8]) -> u32 {
    ((a[0] as u32) << 24)
    + ((a[1] as u32) << 16)
    + ((a[2] as u32) << 8)
    + (a[3] as u32)
}

pub fn make_le32(a: &[u8]) -> u32 {
    ((a[3] as u32) << 24)
    + ((a[2] as u32) << 16)
    + ((a[1] as u32) << 8)
    + (a[0] as u32)
}

pub fn make_le16(a: &[u8]) -> u16 {
    ((a[1] as u16) << 8)
    + (a[0] as u16)
}

pub fn make_be16(a: &[u8]) -> u16 {
    ((a[0] as u16) << 8)
    + (a[1] as u16)
}