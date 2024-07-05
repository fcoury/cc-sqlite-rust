use crate::varint::VarInt;

pub struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        ByteReader { data, pos: 0 }
    }

    pub fn read_u16(&mut self) -> u16 {
        let value = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        value
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.data[self.pos];
        self.pos += 1;
        value
    }

    pub fn read_varint(&mut self) -> u64 {
        let varint = VarInt::new(&self.data[self.pos..]);
        self.pos += varint.bytes as usize;
        u64::from(varint)
    }

    pub fn skip(&mut self, bytes: usize) {
        self.pos += bytes;
    }
}
