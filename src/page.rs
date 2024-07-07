#![allow(dead_code)]
use anyhow::bail;

use crate::byte_reader::ByteReader;
use crate::cell::TableLeafCell;

#[derive(Debug)]
pub enum Page {
    InteriorIndex,
    LeafIndex,
    InteriorTable,
    LeafTable(TableLeafPage),
}

#[derive(Debug)]
pub struct TableLeafPage {
    pub header: LeafPageHeader,
    pub cells: Vec<TableLeafCell>,
}

impl TableLeafPage {
    pub fn new(data: &[u8], offset: Option<u64>) -> anyhow::Result<Self> {
        let mut reader = ByteReader::new(data);
        let header = LeafPageHeader::new(&mut reader);

        let mut cells = Vec::with_capacity(header.cell_count as usize);
        for _ in 0..header.cell_count {
            let cell_ptr = reader.read_u16() as usize;
            let ptr = (cell_ptr as u64 - offset.unwrap_or(0)) as usize;
            let cell = TableLeafCell::new(&data[ptr..])?;
            cells.push(cell);
        }

        Ok(TableLeafPage { header, cells })
    }
}

#[derive(Debug)]
pub struct LeafPageHeader {
    pub freeblock_offset: u16,
    pub cell_count: u16,
    pub cell_content_offset: u16,
    pub num_fragmented_bytes: u8,
}

impl LeafPageHeader {
    pub fn new(reader: &mut ByteReader) -> Self {
        let page_type = reader.read_u8();
        assert_eq!(page_type, 0x0d);

        let freeblock_offset = reader.read_u16(); // first freeblock
        let cell_count = reader.read_u16();
        let cell_content_offset = reader.read_u16();
        let num_fragmented_bytes = reader.read_u8(); // fragmented free bytes

        Self {
            freeblock_offset,
            cell_count,
            cell_content_offset,
            num_fragmented_bytes,
        }
    }
}

pub fn parse_page(data: &[u8], offset: Option<u64>) -> anyhow::Result<Page> {
    match data[0] {
        0x02 => unimplemented!("no interior index b-tree page yet"),
        0x05 => unimplemented!("no interior table b-tree page yet"),
        0x0a => unimplemented!("no index leaf page yet"),
        0x0d => Ok(Page::LeafTable(TableLeafPage::new(data, offset)?)),
        typ => bail!("Unsupported page type: {:x}", typ),
    }
}
