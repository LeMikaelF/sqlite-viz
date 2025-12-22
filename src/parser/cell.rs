use crate::error::{Result, SqliteVizError};
use crate::model::{
    Cell, TableLeafCell, TableInteriorCell, IndexLeafCell, IndexInteriorCell,
    PageType,
};
use crate::parser::varint::{parse_varint, parse_signed_varint};
use crate::parser::record::parse_record;

/// Calculate the maximum local payload for a B-tree cell.
/// This determines how much of the payload is stored in the cell vs overflow pages.
fn max_local_payload(usable_size: u32, is_table_leaf: bool) -> usize {
    if is_table_leaf {
        // For table leaf pages: U - 35
        (usable_size - 35) as usize
    } else {
        // For index pages and table interior: ((U - 12) * 64 / 255) - 23
        (((usable_size - 12) * 64 / 255) - 23) as usize
    }
}

/// Calculate the minimum local payload
fn min_local_payload(usable_size: u32, is_table_leaf: bool) -> usize {
    if is_table_leaf {
        // For table leaf: ((U - 12) * 32 / 255) - 23
        (((usable_size - 12) * 32 / 255) - 23) as usize
    } else {
        // For index pages: ((U - 12) * 32 / 255) - 23
        (((usable_size - 12) * 32 / 255) - 23) as usize
    }
}

/// Calculate how much payload is stored locally in the cell
pub fn calculate_local_payload_size(
    payload_size: u64,
    usable_size: u32,
    is_table_leaf: bool,
) -> usize {
    let max_local = max_local_payload(usable_size, is_table_leaf);
    let min_local = min_local_payload(usable_size, is_table_leaf);

    if payload_size as usize <= max_local {
        // Entire payload fits locally
        payload_size as usize
    } else {
        // Payload overflows - store min_local or enough to fill to overflow boundary
        let overflow_threshold = min_local + ((payload_size as usize - min_local) % (usable_size as usize - 4));
        if overflow_threshold <= max_local {
            overflow_threshold
        } else {
            min_local
        }
    }
}

/// Parse a cell from page data given its offset and page type
pub fn parse_cell(
    page_data: &[u8],
    cell_offset: u16,
    page_type: PageType,
    usable_size: u32,
) -> Result<Cell> {
    let data = &page_data[cell_offset as usize..];

    match page_type {
        PageType::LeafTable => parse_table_leaf_cell(data, cell_offset, usable_size),
        PageType::InteriorTable => parse_table_interior_cell(data, cell_offset),
        PageType::LeafIndex => parse_index_leaf_cell(data, cell_offset, usable_size),
        PageType::InteriorIndex => parse_index_interior_cell(data, cell_offset, usable_size),
        _ => Err(SqliteVizError::InvalidPageType(0)),
    }
}

/// Parse a table B-tree leaf cell (page type 0x0d)
fn parse_table_leaf_cell(data: &[u8], cell_offset: u16, usable_size: u32) -> Result<Cell> {
    let mut offset = 0;

    // Payload size (varint)
    let (payload_size, len1) = parse_varint(data)?;
    offset += len1;

    // Rowid (varint)
    let (rowid, len2) = parse_signed_varint(&data[offset..])?;
    offset += len2;

    // Calculate local payload size
    let local_payload_size = calculate_local_payload_size(payload_size, usable_size, true);

    // Parse payload (if we have enough data)
    let payload = if offset + local_payload_size <= data.len() {
        parse_record(&data[offset..offset + local_payload_size]).ok()
    } else {
        None
    };

    // Check for overflow
    let overflow_page = if payload_size as usize > local_payload_size {
        let overflow_offset = offset + local_payload_size;
        if overflow_offset + 4 <= data.len() {
            Some(u32::from_be_bytes([
                data[overflow_offset],
                data[overflow_offset + 1],
                data[overflow_offset + 2],
                data[overflow_offset + 3],
            ]))
        } else {
            None
        }
    } else {
        None
    };

    // Calculate total cell size
    let cell_size = offset + local_payload_size + if overflow_page.is_some() { 4 } else { 0 };

    Ok(Cell::TableLeaf(TableLeafCell {
        cell_offset,
        cell_size,
        payload_size,
        rowid,
        local_payload_size,
        payload,
        overflow_page,
    }))
}

/// Parse a table B-tree interior cell (page type 0x05)
fn parse_table_interior_cell(data: &[u8], cell_offset: u16) -> Result<Cell> {
    if data.len() < 4 {
        return Err(SqliteVizError::UnexpectedEof { context: "table interior cell" });
    }

    // Left child page (4 bytes big-endian)
    let left_child_page = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

    // Rowid key (varint)
    let (rowid, len) = parse_signed_varint(&data[4..])?;

    let cell_size = 4 + len;

    Ok(Cell::TableInterior(TableInteriorCell {
        cell_offset,
        cell_size,
        left_child_page,
        rowid,
    }))
}

/// Parse an index B-tree leaf cell (page type 0x0a)
fn parse_index_leaf_cell(data: &[u8], cell_offset: u16, usable_size: u32) -> Result<Cell> {
    let mut offset = 0;

    // Payload size (varint)
    let (payload_size, len) = parse_varint(data)?;
    offset += len;

    // Calculate local payload size
    let local_payload_size = calculate_local_payload_size(payload_size, usable_size, false);

    // Parse payload
    let payload = if offset + local_payload_size <= data.len() {
        parse_record(&data[offset..offset + local_payload_size]).ok()
    } else {
        None
    };

    // Check for overflow
    let overflow_page = if payload_size as usize > local_payload_size {
        let overflow_offset = offset + local_payload_size;
        if overflow_offset + 4 <= data.len() {
            Some(u32::from_be_bytes([
                data[overflow_offset],
                data[overflow_offset + 1],
                data[overflow_offset + 2],
                data[overflow_offset + 3],
            ]))
        } else {
            None
        }
    } else {
        None
    };

    let cell_size = offset + local_payload_size + if overflow_page.is_some() { 4 } else { 0 };

    Ok(Cell::IndexLeaf(IndexLeafCell {
        cell_offset,
        cell_size,
        payload_size,
        local_payload_size,
        payload,
        overflow_page,
    }))
}

/// Parse an index B-tree interior cell (page type 0x02)
fn parse_index_interior_cell(data: &[u8], cell_offset: u16, usable_size: u32) -> Result<Cell> {
    if data.len() < 4 {
        return Err(SqliteVizError::UnexpectedEof { context: "index interior cell" });
    }

    // Left child page (4 bytes big-endian)
    let left_child_page = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let mut offset = 4;

    // Payload size (varint)
    let (payload_size, len) = parse_varint(&data[offset..])?;
    offset += len;

    // Calculate local payload size
    let local_payload_size = calculate_local_payload_size(payload_size, usable_size, false);

    // Parse payload
    let payload = if offset + local_payload_size <= data.len() {
        parse_record(&data[offset..offset + local_payload_size]).ok()
    } else {
        None
    };

    // Check for overflow
    let overflow_page = if payload_size as usize > local_payload_size {
        let overflow_offset = offset + local_payload_size;
        if overflow_offset + 4 <= data.len() {
            Some(u32::from_be_bytes([
                data[overflow_offset],
                data[overflow_offset + 1],
                data[overflow_offset + 2],
                data[overflow_offset + 3],
            ]))
        } else {
            None
        }
    } else {
        None
    };

    let cell_size = offset + local_payload_size + if overflow_page.is_some() { 4 } else { 0 };

    Ok(Cell::IndexInterior(IndexInteriorCell {
        cell_offset,
        cell_size,
        left_child_page,
        payload_size,
        local_payload_size,
        payload,
        overflow_page,
    }))
}
