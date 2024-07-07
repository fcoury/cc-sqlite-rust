mod byte_reader;
mod cell;
mod db;
mod page;
mod utils;
mod varint;

use anyhow::{bail, Result};

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
            let db = db::read_db(&args[1])?;
            println!("{:#?}", db);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
