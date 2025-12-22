use serde::Serialize;
use crate::model::cell::Cell;

/// Page type flags as defined in SQLite format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PageType {
    /// Interior index B-tree page (0x02)
    InteriorIndex,
    /// Interior table B-tree page (0x05)
    InteriorTable,
    /// Leaf index B-tree page (0x0a)
    LeafIndex,
    /// Leaf table B-tree page (0x0d)
    LeafTable,
    /// Overflow page
    Overflow,
    /// Freelist trunk page
    FreelistTrunk,
    /// Freelist leaf page
    FreelistLeaf,
    /// Pointer map page (auto-vacuum databases)
    PointerMap,
    /// Lock-byte page (page containing byte at offset 1073741824)
    LockByte,
}

impl PageType {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x02 => Some(PageType::InteriorIndex),
            0x05 => Some(PageType::InteriorTable),
            0x0a => Some(PageType::LeafIndex),
            0x0d => Some(PageType::LeafTable),
            _ => None,
        }
    }

    pub fn is_interior(&self) -> bool {
        matches!(self, PageType::InteriorIndex | PageType::InteriorTable)
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, PageType::LeafIndex | PageType::LeafTable)
    }

    pub fn is_table(&self) -> bool {
        matches!(self, PageType::InteriorTable | PageType::LeafTable)
    }

    pub fn is_index(&self) -> bool {
        matches!(self, PageType::InteriorIndex | PageType::LeafIndex)
    }

    pub fn header_size(&self) -> usize {
        if self.is_interior() { 12 } else { 8 }
    }
}

/// B-tree page header
#[derive(Debug, Clone, Serialize)]
pub struct BTreePageHeader {
    /// Page type
    pub page_type: PageType,
    /// Byte offset to first freeblock (0 = no freeblocks)
    pub first_freeblock: u16,
    /// Number of cells on this page
    pub cell_count: u16,
    /// Byte offset to start of cell content area (0 = 65536)
    pub cell_content_start: u16,
    /// Number of fragmented free bytes in cell content area
    pub fragmented_free_bytes: u8,
    /// Right-most pointer (only for interior pages)
    pub right_most_pointer: Option<u32>,
}

/// Represents a complete parsed page
#[derive(Debug, Clone, Serialize)]
pub struct Page {
    /// Page number (1-indexed)
    pub page_number: u32,
    /// Page type
    pub page_type: PageType,
    /// B-tree header (if this is a B-tree page)
    pub header: Option<BTreePageHeader>,
    /// Cell pointer array (offsets from start of page)
    pub cell_pointers: Vec<u16>,
    /// Parsed cells
    pub cells: Vec<Cell>,
    /// Calculated free space
    pub free_space: usize,
    /// Raw page data
    #[serde(skip)]
    pub raw_data: Vec<u8>,
}

/// Overflow page
#[derive(Debug, Clone, Serialize)]
pub struct OverflowPage {
    /// Page number
    pub page_number: u32,
    /// Next overflow page (0 = end of chain)
    pub next_page: Option<u32>,
    /// Payload content in this overflow page
    pub content_size: usize,
}
