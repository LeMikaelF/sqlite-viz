use serde::Serialize;

/// A cell within a B-tree page
#[derive(Debug, Clone, Serialize)]
pub enum Cell {
    TableLeaf(TableLeafCell),
    TableInterior(TableInteriorCell),
    IndexLeaf(IndexLeafCell),
    IndexInterior(IndexInteriorCell),
}

impl Cell {
    pub fn left_child(&self) -> Option<u32> {
        match self {
            Cell::TableInterior(c) => Some(c.left_child_page),
            Cell::IndexInterior(c) => Some(c.left_child_page),
            _ => None,
        }
    }

    pub fn overflow_page(&self) -> Option<u32> {
        match self {
            Cell::TableLeaf(c) => c.overflow_page,
            Cell::IndexLeaf(c) => c.overflow_page,
            Cell::IndexInterior(c) => c.overflow_page,
            Cell::TableInterior(_) => None,
        }
    }

    pub fn rowid(&self) -> Option<i64> {
        match self {
            Cell::TableLeaf(c) => Some(c.rowid),
            Cell::TableInterior(c) => Some(c.rowid),
            _ => None,
        }
    }

    pub fn payload_size(&self) -> Option<u64> {
        match self {
            Cell::TableLeaf(c) => Some(c.payload_size),
            Cell::IndexLeaf(c) => Some(c.payload_size),
            Cell::IndexInterior(c) => Some(c.payload_size),
            Cell::TableInterior(_) => None,
        }
    }

    /// Get the byte offset where this cell starts in the page
    pub fn cell_offset(&self) -> u16 {
        match self {
            Cell::TableLeaf(c) => c.cell_offset,
            Cell::TableInterior(c) => c.cell_offset,
            Cell::IndexLeaf(c) => c.cell_offset,
            Cell::IndexInterior(c) => c.cell_offset,
        }
    }

    /// Get the total size of this cell in bytes
    pub fn cell_size(&self) -> usize {
        match self {
            Cell::TableLeaf(c) => c.cell_size,
            Cell::TableInterior(c) => c.cell_size,
            Cell::IndexLeaf(c) => c.cell_size,
            Cell::IndexInterior(c) => c.cell_size,
        }
    }
}

/// Table B-tree leaf cell (page type 0x0d)
#[derive(Debug, Clone, Serialize)]
pub struct TableLeafCell {
    /// Offset in page where cell starts
    pub cell_offset: u16,
    /// Total size of cell in bytes
    pub cell_size: usize,
    /// Total payload size (varint)
    pub payload_size: u64,
    /// Row ID (varint)
    pub rowid: i64,
    /// Local payload bytes stored in this cell
    pub local_payload_size: usize,
    /// Parsed payload record
    pub payload: Option<Record>,
    /// First overflow page number (if payload overflows)
    pub overflow_page: Option<u32>,
}

/// Table B-tree interior cell (page type 0x05)
#[derive(Debug, Clone, Serialize)]
pub struct TableInteriorCell {
    /// Offset in page where cell starts
    pub cell_offset: u16,
    /// Total size of cell in bytes
    pub cell_size: usize,
    /// Left child page number (4-byte big-endian)
    pub left_child_page: u32,
    /// Row ID key (varint)
    pub rowid: i64,
}

/// Index B-tree leaf cell (page type 0x0a)
#[derive(Debug, Clone, Serialize)]
pub struct IndexLeafCell {
    /// Offset in page where cell starts
    pub cell_offset: u16,
    /// Total size of cell in bytes
    pub cell_size: usize,
    /// Total payload size (varint)
    pub payload_size: u64,
    /// Local payload bytes stored in this cell
    pub local_payload_size: usize,
    /// Parsed payload record
    pub payload: Option<Record>,
    /// First overflow page number (if payload overflows)
    pub overflow_page: Option<u32>,
}

/// Index B-tree interior cell (page type 0x02)
#[derive(Debug, Clone, Serialize)]
pub struct IndexInteriorCell {
    /// Offset in page where cell starts
    pub cell_offset: u16,
    /// Total size of cell in bytes
    pub cell_size: usize,
    /// Left child page number (4-byte big-endian)
    pub left_child_page: u32,
    /// Total payload size (varint)
    pub payload_size: u64,
    /// Local payload bytes stored in this cell
    pub local_payload_size: usize,
    /// Parsed payload record
    pub payload: Option<Record>,
    /// First overflow page number (if payload overflows)
    pub overflow_page: Option<u32>,
}

/// A parsed record (row payload)
#[derive(Debug, Clone, Serialize)]
pub struct Record {
    /// Header size in bytes
    pub header_size: u64,
    /// Column serial types
    pub column_types: Vec<SerialType>,
    /// Column values
    pub values: Vec<Value>,
}

/// SQLite serial type for a column value
#[derive(Debug, Clone, Copy, Serialize)]
pub enum SerialType {
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
    Reserved(u64),
    Blob(usize),
    Text(usize),
}

impl SerialType {
    pub fn from_raw(value: u64) -> Self {
        match value {
            0 => SerialType::Null,
            1 => SerialType::Int8,
            2 => SerialType::Int16,
            3 => SerialType::Int24,
            4 => SerialType::Int32,
            5 => SerialType::Int48,
            6 => SerialType::Int64,
            7 => SerialType::Float64,
            8 => SerialType::Zero,
            9 => SerialType::One,
            10 | 11 => SerialType::Reserved(value),
            n if n >= 12 && n % 2 == 0 => SerialType::Blob(((n - 12) / 2) as usize),
            n if n >= 13 && n % 2 == 1 => SerialType::Text(((n - 13) / 2) as usize),
            _ => SerialType::Reserved(value),
        }
    }

    /// Get the size in bytes for this serial type
    pub fn size(&self) -> usize {
        match self {
            SerialType::Null => 0,
            SerialType::Int8 => 1,
            SerialType::Int16 => 2,
            SerialType::Int24 => 3,
            SerialType::Int32 => 4,
            SerialType::Int48 => 6,
            SerialType::Int64 => 8,
            SerialType::Float64 => 8,
            SerialType::Zero => 0,
            SerialType::One => 0,
            SerialType::Reserved(_) => 0,
            SerialType::Blob(n) => *n,
            SerialType::Text(n) => *n,
        }
    }
}

/// A parsed column value
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Value {
    /// Get a short preview string for display
    pub fn preview(&self, max_len: usize) -> String {
        match self {
            Value::Null => "NULL".to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Real(f) => format!("{:.6}", f),
            Value::Text(s) => {
                if s.len() <= max_len {
                    format!("\"{}\"", s)
                } else {
                    format!("\"{}...\"", &s[..max_len])
                }
            }
            Value::Blob(b) => {
                if b.len() <= max_len / 2 {
                    format!("x'{}'", hex_encode(b))
                } else {
                    format!("x'{}...' ({} bytes)", hex_encode(&b[..max_len / 2]), b.len())
                }
            }
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
