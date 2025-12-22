use crate::error::{Result, SqliteVizError};
use crate::model::{Page, PageType, BTreePageHeader};
use crate::parser::cell::parse_cell;

/// Parse a B-tree page header
fn parse_btree_header(data: &[u8], page_type: PageType) -> Result<BTreePageHeader> {
    if data.len() < 8 {
        return Err(SqliteVizError::UnexpectedEof { context: "btree page header" });
    }

    let first_freeblock = u16::from_be_bytes([data[1], data[2]]);
    let cell_count = u16::from_be_bytes([data[3], data[4]]);
    let cell_content_start = u16::from_be_bytes([data[5], data[6]]);
    let fragmented_free_bytes = data[7];

    let right_most_pointer = if page_type.is_interior() {
        if data.len() < 12 {
            return Err(SqliteVizError::UnexpectedEof { context: "interior page header" });
        }
        Some(u32::from_be_bytes([data[8], data[9], data[10], data[11]]))
    } else {
        None
    };

    Ok(BTreePageHeader {
        page_type,
        first_freeblock,
        cell_count,
        cell_content_start,
        fragmented_free_bytes,
        right_most_pointer,
    })
}

/// Parse a complete page from raw data
pub fn parse_page(
    page_data: &[u8],
    page_number: u32,
    page_size: u32,
    usable_size: u32,
) -> Result<Page> {
    // Page 1 has 100-byte database header at the start
    let header_offset = if page_number == 1 { 100 } else { 0 };

    if page_data.len() < header_offset + 8 {
        return Err(SqliteVizError::UnexpectedEof { context: "page" });
    }

    let header_data = &page_data[header_offset..];
    let page_type_byte = header_data[0];

    // Check if this is a B-tree page
    let page_type = match PageType::from_byte(page_type_byte) {
        Some(pt) => pt,
        None => {
            // Could be overflow, freelist, or pointer map page
            // For now, we'll handle these separately
            if page_type_byte == 0 {
                // Could be overflow or freelist leaf
                return Ok(Page {
                    page_number,
                    page_type: PageType::Overflow,
                    header: None,
                    cell_pointers: Vec::new(),
                    cells: Vec::new(),
                    free_space: page_size as usize,
                    raw_data: page_data.to_vec(),
                });
            }
            return Err(SqliteVizError::InvalidPageType(page_type_byte));
        }
    };

    // Parse B-tree header
    let header = parse_btree_header(header_data, page_type)?;
    let header_size = page_type.header_size();

    // Parse cell pointer array
    let cell_pointer_start = header_offset + header_size;
    let cell_pointer_end = cell_pointer_start + (header.cell_count as usize * 2);

    if cell_pointer_end > page_data.len() {
        return Err(SqliteVizError::UnexpectedEof { context: "cell pointer array" });
    }

    let mut cell_pointers = Vec::with_capacity(header.cell_count as usize);
    for i in 0..header.cell_count as usize {
        let offset = cell_pointer_start + i * 2;
        let ptr = u16::from_be_bytes([page_data[offset], page_data[offset + 1]]);
        cell_pointers.push(ptr);
    }

    // Parse cells
    let mut cells = Vec::with_capacity(header.cell_count as usize);
    for &ptr in &cell_pointers {
        match parse_cell(page_data, ptr, page_type, usable_size) {
            Ok(cell) => cells.push(cell),
            Err(_) => {
                // Log error but continue parsing other cells
                continue;
            }
        }
    }

    // Calculate free space
    let cell_content_start = if header.cell_content_start == 0 {
        65536
    } else {
        header.cell_content_start as usize
    };

    let _cells_total_size: usize = cells.iter().map(|c| c.cell_size()).sum();
    let free_space = cell_content_start
        .saturating_sub(cell_pointer_end)
        .saturating_add(header.fragmented_free_bytes as usize);

    Ok(Page {
        page_number,
        page_type,
        header: Some(header),
        cell_pointers,
        cells,
        free_space,
        raw_data: page_data.to_vec(),
    })
}

/// Parse an overflow page
pub fn parse_overflow_page(page_data: &[u8], _page_number: u32, usable_size: u32) -> Result<(Option<u32>, usize)> {
    if page_data.len() < 4 {
        return Err(SqliteVizError::UnexpectedEof { context: "overflow page" });
    }

    // First 4 bytes are the next overflow page number (0 = end of chain)
    let next_page = u32::from_be_bytes([page_data[0], page_data[1], page_data[2], page_data[3]]);
    let next_page = if next_page == 0 { None } else { Some(next_page) };

    // Remaining bytes are payload content
    let content_size = usable_size as usize - 4;

    Ok((next_page, content_size))
}
