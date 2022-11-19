use byteorder::ByteOrder;

/// Reads an unsigned 16 bit integer from the buffer and return a generalized usize
pub fn read_u16(buf: &[u8], offset: usize) -> usize {
    byteorder::BigEndian::read_u16(&buf[offset..]).into()
}
