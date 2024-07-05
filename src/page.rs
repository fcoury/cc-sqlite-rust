use std::fmt;

use anyhow::bail;

use crate::byte_reader::ByteReader;

#[derive(Debug)]
enum PageType {
    TableLeaf,
}

impl PageType {
    fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0x0D => Ok(PageType::TableLeaf),
            _ => bail!("Unsupported page type"),
        }
    }
}

#[derive(Debug)]
pub struct Page<'a> {
    typ: PageType,
    cell_count: u16,
    cell_pointer_index: Vec<u16>,
    data: &'a [u8],
    offset: Option<usize>,
}

impl fmt::Display for Page<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Page type: {:?}, cell count: {}, indexes:",
            self.typ, self.cell_count
        )?;
        for idx in &self.cell_pointer_index {
            write!(f, " {:04x}", idx)?
        }
        Ok(())
    }
}

impl<'a> Page<'a> {
    pub fn new(data: &'a [u8], offset: Option<usize>) -> anyhow::Result<Self> {
        // header: 0d xx xx 00 03 xx xx xx
        let mut reader = ByteReader::new(data);

        let typ = PageType::from_u8(reader.read_u8())?;
        reader.skip(2);

        let cell_count = reader.read_u16();
        reader.skip(3);

        let mut cell_pointer_index = Vec::new();
        for _ in 0..cell_count {
            let cell_pointer = reader.read_u16();
            cell_pointer_index.push(cell_pointer);
        }

        Ok(Page {
            typ,
            cell_count,
            cell_pointer_index,
            data,
            offset,
        })
    }

    pub fn cells(&self) -> Vec<Cell> {
        let mut cells = Vec::new();
        for i in 0..self.cell_count {
            let idx = self.cell_pointer_index[i as usize] - self.offset.unwrap_or(0) as u16;
            let cell_data = &self.data[idx as usize..];
            cells.push(Cell::new(cell_data));
        }
        cells
    }
}

#[derive(Debug)]
pub struct Cell<'a> {
    row_id: u64,
    data: &'a [u8],
}

impl<'a> Cell<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let mut reader = ByteReader::new(data);
        let bytes = reader.read_varint();
        let row_id = reader.read_varint();

        Cell {
            row_id,
            data: &data[..bytes as usize],
        }
    }
}
