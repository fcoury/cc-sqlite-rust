#![allow(dead_code)]
use std::io::Read;

use crate::{
    byte_reader::ByteReader,
    page::{parse_page, Page},
    utils::hexdump,
};

#[derive(Debug)]
pub struct Database {
    pub header: DbHeader,
    pub pages: Vec<Page>,
}

#[derive(Debug)]
pub struct DbHeader {
    magic: [u8; 16],
    page_size: u16,
    file_format_write_version: u8,
    file_format_read_version: u8,
    reserved_space: u8,
    max_payload_fraction: u8,
    min_payload_fraction: u8,
    leaf_payload_fraction: u8,
    change_counter: u32,
    in_header_database_size: u32,
    first_freelist_page: u32,
    freelist_pages: u32,
    schema_cookie: u32,
    schema_format: u32,
    cache_size: u32,
    largest_root_btree_page: u32,
    text_encoding: u32,
    user_version: u32,
    incremental_vacuum: u32,
    application_id: u32,
    version_valid_for: u32,
    sqlite_version: u32,
}

impl DbHeader {
    fn new(reader: &mut ByteReader) -> anyhow::Result<Self> {
        let magic = reader.read_bytes(16);
        if magic != b"SQLite format 3\0" {
            anyhow::bail!("Invalid SQLite file format");
        }

        let page_size = reader.read_u16();
        let file_format_write_version = reader.read_u8();
        let file_format_read_version = reader.read_u8();
        let reserved_space = reader.read_u8();
        let max_payload_fraction = reader.read_u8();
        let min_payload_fraction = reader.read_u8();
        let leaf_payload_fraction = reader.read_u8();
        let change_counter = reader.read_u32();
        let in_header_database_size = reader.read_u32();
        let first_freelist_page = reader.read_u32();
        let freelist_pages = reader.read_u32();
        let schema_cookie = reader.read_u32();
        let schema_format = reader.read_u32();
        let cache_size = reader.read_u32();
        let largest_root_btree_page = reader.read_u32();
        let text_encoding = reader.read_u32();
        let user_version = reader.read_u32();
        let incremental_vacuum = reader.read_u32();
        let application_id = reader.read_u32();
        reader.skip(20);
        let version_valid_for = reader.read_u32();
        let sqlite_version = reader.read_u32();

        let magic: [u8; 16] = magic.try_into()?;

        Ok(Self {
            magic,
            page_size,
            file_format_write_version,
            file_format_read_version,
            reserved_space,
            max_payload_fraction,
            min_payload_fraction,
            leaf_payload_fraction,
            change_counter,
            in_header_database_size,
            first_freelist_page,
            freelist_pages,
            schema_cookie,
            schema_format,
            cache_size,
            largest_root_btree_page,
            text_encoding,
            user_version,
            incremental_vacuum,
            application_id,
            version_valid_for,
            sqlite_version,
        })
    }
}

pub fn read_db(file_path: &str) -> anyhow::Result<Database> {
    let mut file = std::fs::File::open(file_path)?;

    let mut header = [0; 100];
    file.read_exact(&mut header)?;

    let mut reader = ByteReader::new(&header);
    let header = DbHeader::new(&mut reader)?;

    println!("{:#?}", header);

    let mut page_data = vec![0; (header.page_size - 100) as usize];
    file.read_exact(&mut page_data)?;

    hexdump(&page_data);
    println!("page data len: {}", page_data.len());

    let page = parse_page(&page_data, Some(100))?;

    Ok(Database {
        header,
        pages: vec![page],
    })
}
