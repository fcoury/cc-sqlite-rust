use crate::varint::VarInt;

pub struct ByteReader<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        ByteReader { data, pos: 0 }
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.data[self.pos];
        self.pos += 1;
        value
    }

    pub fn read_u16(&mut self) -> u16 {
        let value = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        value
    }

    pub fn read_u24(&mut self) -> u32 {
        let value = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            0,
        ]);
        self.pos += 3;
        value
    }

    pub fn read_u32(&mut self) -> u32 {
        let value = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        value
    }

    pub fn read_u48(&mut self) -> u64 {
        let value = u64::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            0,
            0,
        ]);
        self.pos += 6;
        value
    }

    pub fn read_u64(&mut self) -> u64 {
        let value = u64::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ]);
        self.pos += 8;
        value
    }

    pub fn read_f64(&mut self) -> f64 {
        let value = f64::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ]);
        self.pos += 8;
        value
    }

    pub fn read_bytes(&mut self, length: usize) -> &'a [u8] {
        let start = self.pos;
        self.pos += length;
        &self.data[start..self.pos]
    }

    #[allow(unused)]
    pub fn read_varint_with_size(&mut self) -> (u64, u8) {
        let varint = VarInt::new(&self.data[self.pos..]);
        self.pos += varint.len as usize;
        let size = varint.len;
        (u64::from(varint), size)
    }

    pub fn read_varint(&mut self) -> u64 {
        let varint = VarInt::new(&self.data[self.pos..]);
        self.pos += varint.len as usize;
        u64::from(varint)
    }

    #[allow(unused)]
    pub fn peek_u8(&self) -> u8 {
        self.data[self.pos]
    }

    pub fn skip(&mut self, bytes: usize) {
        self.pos += bytes;
    }
}
