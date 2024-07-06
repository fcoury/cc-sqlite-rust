pub fn hexdump(data: &[u8]) {
    for (i, chunk) in data.chunks(16).enumerate() {
        // Print the offset in the first column
        print!("{:08x}  ", i * 16);

        // Print the hexadecimal bytes in the next columns
        for byte in chunk {
            print!("{:02x} ", byte);
        }

        // If the chunk is less than 16 bytes, print extra spaces to align the ASCII column
        if chunk.len() < 16 {
            for _ in 0..(16 - chunk.len()) {
                print!("   ");
            }
        }

        // Print the ASCII representation of the bytes
        print!(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() || byte.is_ascii_whitespace() && !byte == 0x0a {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}
