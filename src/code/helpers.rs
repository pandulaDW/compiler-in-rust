use byteorder::ByteOrder;

/// Reads an unsigned 8 bit value from the buffer and return a general usize.
pub fn read_u8(buf: &[u8]) -> usize {
    buf[0].try_into().unwrap()
}

/// Reads an unsigned 16 bit value from the buffer and return a general usize.
pub fn read_u16(buf: &[u8]) -> usize {
    byteorder::BigEndian::read_u16(&buf[0..]).into()
}
