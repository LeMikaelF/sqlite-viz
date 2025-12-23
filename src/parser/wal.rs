//! WAL (Write-Ahead Log) parsing functionality.

use crate::error::{Result, SqliteVizError};
use crate::model::{
    WalFile, WalFrame, WalFrameHeader, WalHeader, WAL_MAGIC_BIG_ENDIAN, WAL_MAGIC_LITTLE_ENDIAN,
};
use crate::parser::page::parse_page;

/// WAL header size in bytes
pub const WAL_HEADER_SIZE: usize = 32;
/// WAL frame header size in bytes
pub const WAL_FRAME_HEADER_SIZE: usize = 24;

/// Check if data starts with WAL magic bytes
pub fn is_wal_file(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    magic == WAL_MAGIC_BIG_ENDIAN || magic == WAL_MAGIC_LITTLE_ENDIAN
}

/// Parse WAL header from first 32 bytes
pub fn parse_wal_header(data: &[u8]) -> Result<WalHeader> {
    if data.len() < WAL_HEADER_SIZE {
        return Err(SqliteVizError::UnexpectedEof {
            context: "WAL header",
        });
    }

    let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

    // Validate magic
    if magic != WAL_MAGIC_BIG_ENDIAN && magic != WAL_MAGIC_LITTLE_ENDIAN {
        return Err(SqliteVizError::InvalidWalMagic(magic));
    }

    let format_version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    let page_size = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    let checkpoint_sequence = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);
    let salt1 = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let salt2 = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    let checksum1 = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);
    let checksum2 = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);

    Ok(WalHeader {
        magic,
        format_version,
        page_size,
        checkpoint_sequence,
        salt1,
        salt2,
        checksum1,
        checksum2,
    })
}

/// Parse a single WAL frame header
pub fn parse_wal_frame_header(data: &[u8]) -> Result<WalFrameHeader> {
    if data.len() < WAL_FRAME_HEADER_SIZE {
        return Err(SqliteVizError::UnexpectedEof {
            context: "WAL frame header",
        });
    }

    Ok(WalFrameHeader {
        page_number: u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
        db_size_after_commit: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
        salt1: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
        salt2: u32::from_be_bytes([data[12], data[13], data[14], data[15]]),
        checksum1: u32::from_be_bytes([data[16], data[17], data[18], data[19]]),
        checksum2: u32::from_be_bytes([data[20], data[21], data[22], data[23]]),
    })
}

/// Parse an entire WAL file
pub fn parse_wal_file(data: &[u8], file_name: String) -> Result<WalFile> {
    let header = parse_wal_header(data)?;
    let page_size = header.page_size as usize;
    let frame_size = WAL_FRAME_HEADER_SIZE + page_size;

    // For WAL frames, we assume usable_size = page_size (no reserved bytes)
    let usable_size = header.page_size;

    let mut frames = Vec::new();
    let mut offset = WAL_HEADER_SIZE;
    let mut frame_index = 0;

    while offset + frame_size <= data.len() {
        let frame_header = parse_wal_frame_header(&data[offset..])?;

        // Validate salt values match header (frame is valid)
        if frame_header.salt1 != header.salt1 || frame_header.salt2 != header.salt2 {
            // Invalid frame - stop parsing
            break;
        }

        let page_data_start = offset + WAL_FRAME_HEADER_SIZE;
        let page_data = &data[page_data_start..page_data_start + page_size];

        // Parse the page content
        // Use the DB page number from the frame header for correct page 1 handling
        let page = parse_page(
            page_data,
            frame_header.page_number,
            header.page_size,
            usable_size,
        )
        .ok(); // Convert errors to None since some frames may have unparseable pages

        frames.push(WalFrame {
            frame_index,
            header: frame_header,
            page,
            raw_page_data: page_data.to_vec(),
        });

        offset += frame_size;
        frame_index += 1;
    }

    Ok(WalFile {
        header,
        frames,
        file_name,
    })
}
