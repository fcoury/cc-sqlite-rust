#![allow(dead_code)]
use std::io::Read;

use crate::{
    byte_reader::ByteReader,
    page::{parse_page, Page},
};

#[derive(Debug)]
pub struct Database {
    pub header: DbHeader,
    pub pages: Vec<Page>,
}

impl Database {
    pub fn get_page(&self, idx: usize) -> Option<&Page> {
        if idx >= self.pages.len() {
            return None;
        }
        Some(&self.pages[idx])
    }
}

#[derive(Debug)]
pub struct DbHeader {
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

        Ok(Self {
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

    let mut page_data = vec![0; (header.page_size - 100) as usize];
    file.read_exact(&mut page_data)?;

    let mut pages = vec![parse_page(&page_data, Some(100))?];

    loop {
        let mut page_data = vec![0; header.page_size as usize];
        match file.read_exact(&mut page_data) {
            Ok(_) => pages.push(parse_page(&page_data, None)?),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    Ok(Database { header, pages })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::tempdir;

    use crate::cell::{ColType, ColValue};

    use super::*;

    #[test]
    fn empty_db() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.sqlite3");
        let conn = Connection::open(&path).unwrap();

        conn.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, foo TEXT NOT NULL)",
            (),
        )
        .unwrap();
        conn.close().unwrap();

        let db = read_db(&path.to_string_lossy()).unwrap();
        let Page::LeafTable(page) = db.get_page(0).unwrap() else {
            panic!("Expected TableLeaf, got {:?}", db.get_page(0));
        };

        assert_eq!(page.header.cell_count, 1);
        assert_eq!(page.cells.len(), 1);

        let cell = page.cells.first().unwrap();
        assert_eq!(
            cell.column_types,
            vec![
                ColType::Text(5),
                ColType::Text(4),
                ColType::Text(4),
                ColType::Int8,
                ColType::Text(61)
            ]
        );
        assert_eq!(
            cell.column_values,
            vec![
                Some(ColValue::Text("table".to_string())),
                Some(ColValue::Text("test".to_string())),
                Some(ColValue::Text("test".to_string())),
                Some(ColValue::Int8(2)),
                Some(ColValue::Text(
                    "CREATE TABLE test (id INTEGER PRIMARY KEY, foo TEXT NOT NULL)".to_string()
                )),
            ]
        );
    }

    #[test]
    fn parse_table_content() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.sqlite3");
        let conn = Connection::open(&path).unwrap();
        conn.execute(
            "CREATE TABLE test (id INTEGER PRIMARY KEY, foo TEXT NOT NULL)",
            (),
        )
        .unwrap();
        conn.execute("INSERT INTO test VALUES (42, 'tjena tjena')", ())
            .unwrap();
        conn.close().unwrap();

        let db = read_db(&path.to_string_lossy()).unwrap();
        let Page::LeafTable(page) = db.get_page(1).unwrap() else {
            panic!("Expected TableLeaf, got {:?}", db.get_page(1));
        };

        assert_eq!(page.header.cell_count, 1);
        assert_eq!(page.cells.len(), 1);

        let cell = page.cells.first().unwrap();
        assert_eq!(cell.rowid, 42);
        assert_eq!(cell.column_types, vec![ColType::Null, ColType::Text(11)]);
        assert_eq!(
            cell.column_values,
            vec![None, Some(ColValue::Text("tjena tjena".to_string()))]
        );
    }
}
