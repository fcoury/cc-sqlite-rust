use std::fmt;

#[derive(Debug)]
pub struct VarInt {
    value: [u8; 10],
    pub bytes: u8,
}

impl fmt::Display for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.bytes {
            write!(f, "{:02x} ", self.value[9 - (i as usize)])?;
        }
        Ok(())
    }
}

impl VarInt {
    pub fn new(data: &[u8]) -> Self {
        let mut value = [0; 10];
        let mut bytes = 0;

        loop {
            let d = data[bytes as usize];
            value[9 - (bytes as usize)] = d;
            bytes += 1;
            if (d & 0x80) == 0 {
                break;
            }
        }
        assert!(bytes <= 9);

        VarInt { bytes, value }
    }
}

impl From<u64> for VarInt {
    fn from(from_value: u64) -> Self {
        let mut from_value = from_value;

        // checks if the value has bits after the upper 32 bits
        if from_value & 0xFF_00_00_00 == 0 {}

        let mut bytes: u8 = 0;
        let mut value = [0; 10];
        loop {
            value[bytes as usize] = ((from_value & 0x7F) | 0x80) as u8;
            from_value >>= 7;
            bytes += 1;
            if from_value == 0 {
                break;
            }
        }
        assert!(bytes <= 9);

        // Clears the continuation bit on the first byte
        value[0] &= 0x7F;
        value.reverse();

        VarInt { value, bytes }
    }
}

impl From<VarInt> for u64 {
    fn from(from_value: VarInt) -> Self {
        let mut result = 0;
        let mut shift = 0;

        for i in 0..from_value.bytes {
            result |= ((from_value.value[9 - (i as usize)] & 0x7F) as u64) << shift;
            shift += 7;
        }

        result
    }
}

#[test]
fn test_varint_single() {
    let varint = VarInt::from(1935);
    let value: u64 = varint.into();
    assert_eq!(value, 1935);
}

#[test]
fn test_varint_64bit() {
    let varint = VarInt::from(0x7F_FF_FF_FF_FF_FF_FF_FF);
    let value: u64 = varint.into();
    assert_eq!(value, 0x7F_FF_FF_FF_FF_FF_FF_FF);
}
