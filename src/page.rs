use anyhow::bail;

use crate::byte_reader::ByteReader;

pub fn parse_page(data: &[u8]) -> anyhow::Result<Page> {
    let typ = data[0];

    match typ {
        0x02 => unimplemented!("no interior index b-tree page yet"),
        0x05 => unimplemented!("no interior table b-tree page yet"),
        0x0a => unimplemented!("no index leaf page yet"),
        0x0d => Ok(Page::TableLeaf(TableLeafPage::new(data, Some(100))?)),
        _ => bail!("Unsupported page type: {:x}", typ),
    }
}

#[derive(Debug)]
pub enum Page {
    IndexInterior,
    IndexLeaf,
    TableInterior,
    TableLeaf(TableLeafPage),
}

#[derive(Debug)]
pub struct TableLeafPage {
    cells: Vec<TableLeafCell>,
}

impl TableLeafPage {
    pub fn new(data: &[u8], offset: Option<usize>) -> anyhow::Result<Self> {
        let mut reader = ByteReader::new(data);

        reader.skip(1); // page type
        reader.skip(2); // first freeblock

        let cell_count = reader.read_u16();
        let cell_content_area = reader.read_u16();
        println!("cell_content_area: {:4x}", cell_content_area);

        reader.skip(1); // fragmented free bytes

        let mut cells = Vec::new();
        for _ in 0..cell_count {
            let cell_pointer = reader.read_u16();
            let ptr = (cell_pointer - offset.unwrap_or(0) as u16) as usize;
            cells.push(parse_cell(&data[ptr..])?);
        }

        Ok(TableLeafPage { cells })
    }
}

#[derive(Debug)]
enum ColType {
    Null,
    Int8,
    Int16,
    Int24,
    Int32,
    Int48,
    Int64,
    Float64,
    Zero,
    One,
    Blob(u8),
    Text(u8),
}

fn parse_cell(data: &[u8]) -> anyhow::Result<TableLeafCell> {
    let mut reader = ByteReader::new(data);

    // --- cell info section ---

    let payload_size = reader.read_varint(); // payload bytes
    println!("payload_size: {}", payload_size);
    let row_id = reader.read_varint(); // rowid

    // --- header section ---

    let (header_size, size) = reader.read_varint_with_size();
    let remaining_header_size = header_size - size as u64;

    // --- header columns ---

    let mut columns = Vec::new();
    while reader.pos < remaining_header_size as usize {
        // A record contains a header and a body, in that order. The header begins with a single
        // varint which determines the total number of bytes in the header. The varint value is the
        // size of the header in bytes including the size varint itself. Following the size varint
        // are one or more additional varints, one per column. These additional varints are called
        // "serial type" numbers and determine the datatype of each column, according to the
        // following chart:

        // The values for each column in the record immediately follow the header. For serial types
        // 0, 8, 9, 12, and 13, the value is zero bytes in length. If all columns are of these
        // types then the body section of the record is empty.

        let col_size = reader.read_varint();
        println!("- col_size: {}", col_size);

        let col_type = reader.read_varint();
        println!("  col_type: {:x}", col_type);

        match col_type {
            0 => {
                println!("  NULL");
                columns.push(ColType::Null);
            }
            1 => {
                println!("  8-bit");
                columns.push(ColType::Int8);
            }
            2 => {
                println!("  16-bit");
                columns.push(ColType::Int16);
            }
            3 => {
                println!("  24-bit");
                columns.push(ColType::Int24);
            }
            4 => {
                println!("  32-bit");
                columns.push(ColType::Int32);
            }
            5 => {
                println!("  48-bit");
                columns.push(ColType::Int48);
            }
            6 => {
                println!("  64-bit");
                columns.push(ColType::Int64);
            }
            7 => {
                println!("  64-bit float");
                columns.push(ColType::Float64);
            }
            8 => {
                println!("  Zero");
                columns.push(ColType::Zero);
            }
            9 => {
                println!("  One");
                columns.push(ColType::One);
            }
            10..=11 => {
                println!("  Internal use");
            }
            12.. if col_type % 2 == 0 => {
                let length = (col_type - 12) / 2;
                println!("  BLOB length: {}", length);
                columns.push(ColType::Blob(length as u8));
            }
            13.. if col_type % 2 == 1 => {
                let length = (col_type - 13) / 2;
                println!("  TEXT length: {}", length);
                columns.push(ColType::Text(length as u8));
            }
            _ => bail!("Unsupported column type: {}", col_type),
        }
        println!();
    }

    println!("Columns: {:?}", columns);

    Ok(TableLeafCell { row_id })
}

#[derive(Debug)]
struct TableLeafCell {
    row_id: u64,
}

/*
impl PageType {
    fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0x0D => Ok(PageType::TableLeaf),
            _ => bail!("Unsupported page type"),
        }
    }
}

#[derive(Debug)]
pub struct TableLeafPageOld<'a> {
    typ: PageType,
    cell_count: u16,
    cell_pointer_index: Vec<u16>,
    data: &'a [u8],
    offset: Option<usize>,
}

impl fmt::Display for TableLeafPageOld<'_> {
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

impl<'a> TableLeafPageOld<'a> {
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

        Ok(TableLeafPageOld {
            typ,
            cell_count,
            cell_pointer_index,
            data,
            offset,
        })
    }

    pub fn cells(&self) -> anyhow::Result<Vec<TableLeafCellOld>> {
        let mut cells = Vec::new();
        for i in 0..self.cell_count {
            let idx = self.cell_pointer_index[i as usize] - self.offset.unwrap_or(0) as u16;
            let cell_data = &self.data[idx as usize..];
            cells.push(TableLeafCellOld::new(cell_data)?);
        }
        Ok(cells)
    }
}

#[derive(Debug)]
pub struct TableLeafCellOld<'a> {
    row_id: u64,
    data: &'a [u8],
}

impl<'a> TableLeafCellOld<'a> {
    pub fn new(data: &'a [u8]) -> anyhow::Result<Self> {
        let mut reader = ByteReader::new(data);
        let bytes = reader.read_varint();
        println!("bytes: {}", bytes);
        let row_id = reader.read_varint();
        println!("row_id: {}", row_id);

        let (header_size, size) = reader.read_varint_with_size();
        println!("header_size: {}", header_size);

        let max_size = header_size - size as u64;
        println!("\nColumns:");

        while reader.pos < max_size as usize {
            let col_size = reader.read_varint();
            println!("- col_size: {}", col_size);

            let col_type = reader.read_varint();
            println!("  col_type: {:x}", col_type);

            match col_type {
                0 => println!("  NULL"),
                1 => println!("  8-bit"),
                2 => println!("  16-bit"),
                3 => println!("  24-bit"),
                4 => println!("  32-bit"),
                5 => println!("  48-bit"),
                6 => println!("  64-bit"),
                7 => println!("  64-bit float"),
                8 => println!("  Zero"),
                9 => println!("  One"),
                10..=11 => println!("  Internal use"),
                12.. if col_type % 2 == 0 => {
                    let length = (col_type - 12) / 2;
                    println!("  BLOB length: {}", length);
                }
                13.. if col_type % 2 == 1 => {
                    let length = (col_type - 13) / 2;
                    println!("  TEXT length: {}", length);
                }
                _ => bail!("Unsupported column type: {}", col_type),
            }
            println!();
        }

        println!();

        Ok(TableLeafCellOld {
            row_id,
            data: &data[..bytes as usize],
        })
    }
}
*/
