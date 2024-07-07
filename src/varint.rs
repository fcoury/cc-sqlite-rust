use std::fmt;

#[derive(Debug)]
pub struct VarInt {
    value: [u8; 10],
    pub len: u8,
}

impl fmt::Display for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.len {
            write!(f, "{:02x} ", self.value[i as usize])?;
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
            value[bytes as usize] = d;
            bytes += 1;
            if (d & 0x80) == 0 {
                break;
            }
        }
        assert!(bytes <= 9);

        VarInt { len: bytes, value }
    }
}

impl From<VarInt> for u64 {
    fn from(varint: VarInt) -> u64 {
        let mut ux: u64 = 0;

        for byte in varint.value.iter() {
            ux = (ux << 7) + (byte & 0x7F) as u64;
            if byte & 0x80 == 0 {
                return ux;
            }
        }

        return ux;
    }
}

impl From<u64> for VarInt {
    fn from(mut v: u64) -> Self {
        let mut value = [0u8; 10];
        let mut n = 0;

        loop {
            value[n] = (v & 0x7f) as u8;
            v >>= 7;
            if v != 0 {
                value[n] |= 0x80;
            }
            n += 1;
            if v == 0 || n == 10 {
                break;
            }
        }

        println!("Encoded value: {:?}", &value[..n]); // Debug print

        VarInt {
            value,
            len: n as u8,
        }
    }
}

#[test]
fn test_varint_single() {
    let varint = VarInt::from(1935);
    let value: u64 = varint.into();
    assert_eq!(value, 1935);
}

#[test]
fn test_varint_from_parts() {
    let varint = VarInt::new(&[0x81, 0x07]);
    let value: u64 = varint.into();
    assert_eq!(value, 135);
}

#[test]
fn test_varint_64bit() {
    let varint = VarInt::from(0x7F_FF_FF_FF_FF_FF_FF_FF);
    let value: u64 = varint.into();
    assert_eq!(value, 0x7F_FF_FF_FF_FF_FF_FF_FF);
}

#[test]
fn test_varint_zero() {
    let varint = VarInt::from(0);
    assert_eq!(varint.len, 1);
}
