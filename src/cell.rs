use crate::byte_reader::ByteReader;

#[derive(Debug)]
pub struct TableLeafCell {
    rowid: u64,
    payload_size: u64,
    column_types: Vec<ColType>,
    column_values: Vec<Option<ColValue>>,
}

impl TableLeafCell {
    pub fn new(data: &[u8]) -> anyhow::Result<Self> {
        let mut reader = ByteReader::new(data);

        let payload_size = reader.read_varint();
        let rowid = reader.read_varint();
        let header_size = reader.read_varint();

        let mut column_types = Vec::new();
        while reader.pos < header_size as usize - 1 {
            let col_type = ColType::from(reader.read_varint());
            column_types.push(col_type);
        }

        let mut reader = ByteReader::new(&data[header_size as usize..]);
        let page_number = reader.read_varint();
        println!("page_number: {:04x}", page_number);
        let mut column_values = Vec::with_capacity(column_types.len());
        for col_type in &column_types {
            let col_value = match col_type {
                ColType::Null => None,
                ColType::Int8 => Some(ColValue::Int8(reader.read_u8())),
                ColType::Int16 => Some(ColValue::Int16(reader.read_u16())),
                ColType::Int24 => Some(ColValue::Int24(reader.read_u24())),
                ColType::Int32 => Some(ColValue::Int32(reader.read_u32())),
                ColType::Int48 => Some(ColValue::Int48(reader.read_u48())),
                ColType::Int64 => Some(ColValue::Int64(reader.read_u64())),
                ColType::Float64 => Some(ColValue::Float64(reader.read_f64())),
                ColType::Zero => Some(ColValue::Zero),
                ColType::One => Some(ColValue::One),
                ColType::Reserved => unimplemented!("reserved column type"),
                ColType::Blob(length) if *length == 0 => None,
                ColType::Blob(length) => {
                    let value = reader.read_bytes(*length as usize);
                    Some(ColValue::Blob(value.to_vec()))
                }
                ColType::Text(length) if *length == 0 => None,
                ColType::Text(length) => {
                    let value = reader.read_bytes(*length as usize);
                    Some(ColValue::Text(String::from_utf8_lossy(&value).to_string()))
                }
            };

            column_values.push(col_value);
        }

        Ok(Self {
            rowid,
            payload_size,
            column_types,
            column_values,
        })
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
    Reserved,
    Blob(u8),
    Text(u8),
}

impl From<u64> for ColType {
    fn from(value: u64) -> Self {
        match value {
            0 => ColType::Null,
            1 => ColType::Int8,
            2 => ColType::Int16,
            3 => ColType::Int24,
            4 => ColType::Int32,
            5 => ColType::Int48,
            6 => ColType::Int64,
            7 => ColType::Float64,
            8 => ColType::Zero,
            9 => ColType::One,
            10..=11 => ColType::Reserved,
            12.. if value % 2 == 0 => ColType::Blob((value as u8 - 12) / 2),
            13.. if value % 2 == 1 => ColType::Text((value as u8 - 13) / 2),
            _ => panic!("Unsupported column type: {}", value),
        }
    }
}

#[derive(Debug)]
enum ColValue {
    Null,
    Int8(u8),
    Int16(u16),
    Int24(u32),
    Int32(u32),
    Int48(u64),
    Int64(u64),
    Float64(f64),
    Zero,
    One,
    Blob(Vec<u8>),
    Text(String),
}
