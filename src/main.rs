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

            // You can use print statements as follows for debugging, they'll be visible when running tests.
            println!("Logs from your program will appear here!");

            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            let page_size = u16::from_be_bytes([header[16], header[17]]);
            println!("database page size: {}", page_size);

            let mut page = vec![0; page_size as usize];
            file.read_exact(&mut page)?;

            let page_type = page[0];
            let number_of_tables = u16::from_be_bytes([page[3], page[4]]);
            let cell_content_offset = u16::from_be_bytes([page[5], page[6]]);

            println!("page_type: 0x{:02x}", page_type);
            println!("number of tables: {}", number_of_tables);

            for i in 0..number_of_tables {
                let cell_pointer_offset = 10 + ((i * 2) as usize);
                let cell_pointer =
                    u16::from_be_bytes([page[cell_pointer_offset], page[cell_pointer_offset + 1]])
                        as usize;
                println!("{} - cell_pointer: {}", i + 1, cell_pointer);

                let (n, payload_size) = parse_varint(&page[cell_pointer..]);
                println!("payload_size: {}", payload_size);
                let (n, row_id) = parse_varint(&page[(cell_pointer + n)..]);
                println!("row_id: {}", row_id);
            }

            // let content = &page[cell_content_offset as usize..];
            // println!("content 0: {:x}", &content[0]);
            // println!("content: {}", String::from_utf8_lossy(content));

            // Table B-Tree Leaf Cell (header 0x0d):
            // A varint which is the total number of bytes of payload, including any overflow
            // A varint which is the integer key, a.k.a. "rowid"
            // The initial portion of the payload that does not spill to overflow pages.
            // A 4-byte big-endian integer page number for the first page of the overflow page list - omitted if all payload fits on the b-tree page.

            // let mut pos = 0;
            // let (size, payload_size) = parse_varint(&content);
            // pos += size;
            // let (size, row_id) = parse_varint(&content[pos..]);
            // pos += size;
            // println!("payload size: {}", payload_size);
            // println!("row id: {}", row_id);
            // let payload = &content[pos..];
            // println!("payload: {}", String::from_utf8_lossy(payload));
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}

fn consume_varint(buf: &mut [u8]) {}

/// A variable-length integer or "varint" is a static Huffman encoding of 64-bit
/// twos-complement integers that uses less space for small positive values. A varint is
/// between 1 and 9 bytes in length. The varint consists of either zero or more bytes
/// which have the high-order bit set followed by a single byte with the high-order bit
/// clear, or nine bytes, whichever is shorter. The lower seven bits of each of the
/// first eight bytes and all 8 bits of the ninth byte are used to reconstruct the
/// 64-bit twos-complement integer. Varints are big-endian: bits taken from the earlier
/// byte of the varint are more significant than bits taken from the later bytes.
fn parse_varint(buf: &[u8]) -> (usize, u64) {
    let mut size = 0;
    let mut values = vec![];
    for i in 0..9 {
        size += 1;
        let byte = buf[i];
        let mask = if i == 8 { 0b1111_1111 } else { 0b0111_1111 };
        let byte_value = (byte & mask) as u64;
        values.push(byte_value);
        if byte & 0b1000_0000 == 0 {
            break;
        }
    }
    assert!(size <= 10);

    let mut value = String::new();
    for (i, v) in values.iter().enumerate() {
        let binary = &if i == 8 {
            format!("{:08b}", v)
        } else {
            format!("{:07b}", v)
        };
        value.push_str(binary);
    }

    (size, u64::from_str_radix(&value, 2).unwrap())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_varint_single() {
        let bytes = [15];
        assert_eq!(parse_varint(&bytes), (1, 15));
    }

    #[test]
    fn test_varint_single_with_padding() {
        let bytes = [15, 0, 0, 0, 0, 0];
        assert_eq!(parse_varint(&bytes), (1, 15));
    }

    #[test]
    fn test_varint_two_bytes() {
        let bytes = [143, 15];
        assert_eq!(parse_varint(&bytes), (2, 1935));
    }

    #[test]
    fn test_varint_large() {
        let bytes = [0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x8f, 0x0f];
        assert_eq!(parse_varint(&bytes), (9, 2178749300044435215));
    }
}
