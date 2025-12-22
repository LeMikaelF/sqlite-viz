use crate::error::{Result, SqliteVizError};
use crate::model::{Record, SerialType, Value};
use crate::parser::varint::parse_varint;

/// Parse a record payload from cell data
pub fn parse_record(data: &[u8]) -> Result<Record> {
    if data.is_empty() {
        return Err(SqliteVizError::UnexpectedEof { context: "record" });
    }

    // Parse header size
    let (header_size, header_size_len) = parse_varint(data)?;

    // Parse serial types from header
    let mut offset = header_size_len;
    let mut column_types = Vec::new();

    while offset < header_size as usize {
        if offset >= data.len() {
            break;
        }
        let (serial_type_raw, len) = parse_varint(&data[offset..])?;
        column_types.push(SerialType::from_raw(serial_type_raw));
        offset += len;
    }

    // Parse values based on serial types
    let mut values = Vec::new();
    let mut value_offset = header_size as usize;

    for serial_type in &column_types {
        if value_offset >= data.len() {
            // Payload may be truncated due to overflow
            values.push(Value::Null);
            continue;
        }

        let remaining = &data[value_offset..];
        let (value, len) = parse_value(remaining, serial_type)?;
        values.push(value);
        value_offset += len;
    }

    Ok(Record {
        header_size,
        column_types,
        values,
    })
}

/// Parse a single value based on its serial type
fn parse_value(data: &[u8], serial_type: &SerialType) -> Result<(Value, usize)> {
    let size = serial_type.size();

    if data.len() < size {
        // Handle truncated data gracefully
        return Ok((Value::Null, 0));
    }

    match serial_type {
        SerialType::Null => Ok((Value::Null, 0)),

        SerialType::Int8 => {
            let val = data[0] as i8 as i64;
            Ok((Value::Integer(val), 1))
        }

        SerialType::Int16 => {
            let val = i16::from_be_bytes([data[0], data[1]]) as i64;
            Ok((Value::Integer(val), 2))
        }

        SerialType::Int24 => {
            // Sign-extend from 24 bits
            let val = if data[0] & 0x80 != 0 {
                // Negative
                let raw = ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | (data[2] as u32);
                (raw | 0xFF000000) as i32 as i64
            } else {
                ((data[0] as i64) << 16) | ((data[1] as i64) << 8) | (data[2] as i64)
            };
            Ok((Value::Integer(val), 3))
        }

        SerialType::Int32 => {
            let val = i32::from_be_bytes([data[0], data[1], data[2], data[3]]) as i64;
            Ok((Value::Integer(val), 4))
        }

        SerialType::Int48 => {
            // Sign-extend from 48 bits
            let raw = ((data[0] as u64) << 40)
                | ((data[1] as u64) << 32)
                | ((data[2] as u64) << 24)
                | ((data[3] as u64) << 16)
                | ((data[4] as u64) << 8)
                | (data[5] as u64);
            let val = if data[0] & 0x80 != 0 {
                (raw | 0xFFFF000000000000) as i64
            } else {
                raw as i64
            };
            Ok((Value::Integer(val), 6))
        }

        SerialType::Int64 => {
            let val = i64::from_be_bytes([
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
            ]);
            Ok((Value::Integer(val), 8))
        }

        SerialType::Float64 => {
            let val = f64::from_be_bytes([
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
            ]);
            Ok((Value::Real(val), 8))
        }

        SerialType::Zero => Ok((Value::Integer(0), 0)),

        SerialType::One => Ok((Value::Integer(1), 0)),

        SerialType::Reserved(_) => Ok((Value::Null, 0)),

        SerialType::Blob(len) => {
            let blob = data[..*len].to_vec();
            Ok((Value::Blob(blob), *len))
        }

        SerialType::Text(len) => {
            let text = String::from_utf8_lossy(&data[..*len]).to_string();
            Ok((Value::Text(text), *len))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_record() {
        // A simple record with header size 2, one NULL column
        let data = [0x02, 0x00]; // header_size=2, serial_type=0 (NULL)
        let record = parse_record(&data).unwrap();
        assert_eq!(record.header_size, 2);
        assert_eq!(record.column_types.len(), 1);
        assert!(matches!(record.column_types[0], SerialType::Null));
    }
}
