mod byte_reader;
mod page;
mod varint;

use anyhow::{bail, Result};
use page::{parse_page, Page};
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

            let mut page_data = vec![0; (page_size - 100) as usize];
            file.read_exact(&mut page_data)?;

            let page = parse_page(&page_data)?;

            let Page::TableLeaf(page) = page else {
                bail!("Invalid page type");
            };

            // let page = TableLeafPageOld::new(&page_data, Some(100))?;
            println!("page: {:?}", page);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
