use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut header = [0; 100];
            file.read_exact(&mut header)?;

            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            let page_size = u16::from_be_bytes([header[16], header[17]]);

            // You can use print statements as follows for debugging, they'll be visible when running tests.
            println!("Logs from your program will appear here!");

            println!("database page size: {}", page_size);

            let mut page = vec![0; page_size as usize];
            file.read_exact(&mut page)?;

            let page_type = page[0];
            println!("page type: {}", page_type);
            let number_of_tables = u16::from_be_bytes([page[3], page[4]]);
            println!("number of tables: {}", number_of_tables);

            // let mut table_names = vec![];

            let content_area_start = u16::from_be_bytes([page[5], page[6]]);
            println!("content area start: {}", content_area_start);
            let content = &page[content_area_start as usize..];

            // A variable-length integer or "varint" is a static Huffman encoding of 64-bit
            // twos-complement integers that uses less space for small positive values. A varint is
            // between 1 and 9 bytes in length. The varint consists of either zero or more bytes
            // which have the high-order bit set followed by a single byte with the high-order bit
            // clear, or nine bytes, whichever is shorter. The lower seven bits of each of the
            // first eight bytes and all 8 bits of the ninth byte are used to reconstruct the
            // 64-bit twos-complement integer. Varints are big-endian: bits taken from the earlier
            // byte of the varint are more significant than bits taken from the later bytes.
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}

fn get_varint(buf: &[u8]) -> u64 {
    let mut values = vec![];
    for i in 0..9 {
        let byte = buf[i];
        let mask = if i == 8 { 0b1111_1111 } else { 0b0111_1111 };
        let byte_value = (byte & mask) as u64;
        values.push(byte_value);
        if byte & 0b1000_0000 == 0 {
            break;
        }
    }

    let mut value = String::new();
    for (i, v) in values.iter().enumerate() {
        let binary = &if i == 8 {
            format!("{:08b}", v)
        } else {
            format!("{:07b}", v)
        };
        value.push_str(binary);
    }

    u64::from_str_radix(&value, 2).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_varint_single() {
        let bytes = [15];
        assert_eq!(get_varint(&bytes), 15);
    }

    #[test]
    fn test_varint_two_bytes() {
        let bytes = [143, 15];
        assert_eq!(get_varint(&bytes), 1935);
    }

    #[test]
    fn test_varint_large() {
        let bytes = [0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x0f];
        assert_eq!(get_varint(&bytes), 2178749300044435215);
    }
}
