use crate::error::{Result, SqliteVizError};

/// Parse a SQLite varint (1-9 bytes, big-endian, 7 bits per byte).
/// High bit set means more bytes follow.
/// Returns (value, bytes_consumed).
pub fn parse_varint(data: &[u8]) -> Result<(u64, usize)> {
    if data.is_empty() {
        return Err(SqliteVizError::InvalidVarint);
    }

    let mut result: u64 = 0;
    let mut bytes_read = 0;

    for (i, &byte) in data.iter().take(9).enumerate() {
        bytes_read = i + 1;

        if i == 8 {
            // 9th byte uses all 8 bits
            result = (result << 8) | (byte as u64);
            break;
        }

        result = (result << 7) | ((byte & 0x7F) as u64);

        if byte & 0x80 == 0 {
            break;
        }
    }

    Ok((result, bytes_read))
}

/// Parse a signed varint. SQLite uses two's complement for negative values.
pub fn parse_signed_varint(data: &[u8]) -> Result<(i64, usize)> {
    let (value, len) = parse_varint(data)?;
    Ok((value as i64, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_byte_varint() {
        // Values 0-127 are single byte
        assert_eq!(parse_varint(&[0x00]).unwrap(), (0, 1));
        assert_eq!(parse_varint(&[0x01]).unwrap(), (1, 1));
        assert_eq!(parse_varint(&[0x7F]).unwrap(), (127, 1));
    }

    #[test]
    fn test_two_byte_varint() {
        // 128 = 0x80 0x00 (continuation bit set on first byte)
        assert_eq!(parse_varint(&[0x81, 0x00]).unwrap(), (128, 2));
        // 300 = 0x82 0x2C
        assert_eq!(parse_varint(&[0x82, 0x2C]).unwrap(), (300, 2));
    }

    #[test]
    fn test_empty_input() {
        assert!(parse_varint(&[]).is_err());
    }
}
