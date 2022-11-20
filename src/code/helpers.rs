use byteorder::ByteOrder;

/// Reads an unsigned 16 bit integer from the buffer and return a general usize.
pub fn read_u16(buf: &[u8]) -> usize {
    byteorder::BigEndian::read_u16(&buf[0..]).into()
}
