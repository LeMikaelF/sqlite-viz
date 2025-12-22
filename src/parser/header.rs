use crate::error::{Result, SqliteVizError};
use crate::model::{DatabaseHeader, TextEncoding};

const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// Parse the 100-byte SQLite database header
pub fn parse_database_header(data: &[u8]) -> Result<DatabaseHeader> {
    if data.len() < 100 {
        return Err(SqliteVizError::UnexpectedEof { context: "database header" });
    }

    // Verify magic header
    if &data[0..16] != SQLITE_MAGIC {
        return Err(SqliteVizError::InvalidMagic);
    }

    // Page size (bytes 16-17)
    let page_size_raw = u16::from_be_bytes([data[16], data[17]]);
    let page_size = if page_size_raw == 1 { 65536 } else { page_size_raw as u32 };

    // File format versions (bytes 18-19)
    let file_format_write = data[18];
    let file_format_read = data[19];

    // Reserved bytes per page (byte 20)
    let reserved_bytes_per_page = data[20];

    // Payload fractions (bytes 21-23)
    let max_payload_fraction = data[21];
    let min_payload_fraction = data[22];
    let leaf_payload_fraction = data[23];

    // File change counter (bytes 24-27)
    let file_change_counter = u32::from_be_bytes([data[24], data[25], data[26], data[27]]);

    // Database size in pages (bytes 28-31)
    let database_size_pages = u32::from_be_bytes([data[28], data[29], data[30], data[31]]);

    // First freelist trunk page (bytes 32-35)
    let first_freelist_page = u32::from_be_bytes([data[32], data[33], data[34], data[35]]);

    // Total freelist pages (bytes 36-39)
    let freelist_page_count = u32::from_be_bytes([data[36], data[37], data[38], data[39]]);

    // Schema cookie (bytes 40-43)
    let schema_cookie = u32::from_be_bytes([data[40], data[41], data[42], data[43]]);

    // Schema format number (bytes 44-47)
    let schema_format = u32::from_be_bytes([data[44], data[45], data[46], data[47]]);

    // Default page cache size (bytes 48-51)
    let default_cache_size = u32::from_be_bytes([data[48], data[49], data[50], data[51]]);

    // Largest root btree page (bytes 52-55) - auto-vacuum
    let largest_root_page = u32::from_be_bytes([data[52], data[53], data[54], data[55]]);

    // Text encoding (bytes 56-59)
    let text_encoding_raw = u32::from_be_bytes([data[56], data[57], data[58], data[59]]);
    let text_encoding = TextEncoding::from_u32(text_encoding_raw)
        .ok_or(SqliteVizError::InvalidTextEncoding(text_encoding_raw))?;

    // User version (bytes 60-63)
    let user_version = u32::from_be_bytes([data[60], data[61], data[62], data[63]]);

    // Incremental vacuum mode (bytes 64-67)
    let incremental_vacuum = u32::from_be_bytes([data[64], data[65], data[66], data[67]]);

    // Application ID (bytes 68-71)
    let application_id = u32::from_be_bytes([data[68], data[69], data[70], data[71]]);

    // Reserved for expansion (bytes 72-91) - skip

    // Version-valid-for (bytes 92-95)
    let version_valid_for = u32::from_be_bytes([data[92], data[93], data[94], data[95]]);

    // SQLite version number (bytes 96-99)
    let sqlite_version = u32::from_be_bytes([data[96], data[97], data[98], data[99]]);

    Ok(DatabaseHeader {
        page_size,
        file_format_write,
        file_format_read,
        reserved_bytes_per_page,
        max_payload_fraction,
        min_payload_fraction,
        leaf_payload_fraction,
        file_change_counter,
        database_size_pages,
        first_freelist_page,
        freelist_page_count,
        schema_cookie,
        schema_format,
        default_cache_size,
        largest_root_page,
        text_encoding,
        user_version,
        incremental_vacuum,
        application_id,
        version_valid_for,
        sqlite_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_magic() {
        let data = [0u8; 100];
        assert!(matches!(parse_database_header(&data), Err(SqliteVizError::InvalidMagic)));
    }

    #[test]
    fn test_too_short() {
        let data = [0u8; 50];
        assert!(matches!(parse_database_header(&data), Err(SqliteVizError::UnexpectedEof { .. })));
    }
}
