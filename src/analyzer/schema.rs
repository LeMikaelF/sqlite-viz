use crate::error::{Result, SqliteVizError};
use crate::model::{Schema, SchemaEntry, ObjectType, Page, Cell, Value, PageType};

/// Parse the sqlite_schema table from page 1 and build the schema
pub fn parse_schema(page1: &Page) -> Result<Schema> {
    let mut schema = Schema::new();

    // sqlite_schema is always a table B-tree starting at page 1
    if page1.page_type != PageType::LeafTable && page1.page_type != PageType::InteriorTable {
        return Err(SqliteVizError::SchemaError(
            "Page 1 is not a table B-tree page".to_string()
        ));
    }

    // For now, we only handle the case where sqlite_schema fits in page 1 (leaf)
    // A more complete implementation would traverse interior nodes
    if page1.page_type == PageType::LeafTable {
        for cell in &page1.cells {
            if let Cell::TableLeaf(leaf_cell) = cell {
                if let Some(record) = &leaf_cell.payload {
                    if let Some(entry) = parse_schema_record(record) {
                        schema.entries.push(entry);
                    }
                }
            }
        }
    }

    Ok(schema)
}

/// Parse a single schema record from sqlite_schema
/// Columns: type, name, tbl_name, rootpage, sql
fn parse_schema_record(record: &crate::model::Record) -> Option<SchemaEntry> {
    if record.values.len() < 5 {
        return None;
    }

    // type (text)
    let object_type = match &record.values[0] {
        Value::Text(s) => ObjectType::from_str(s)?,
        _ => return None,
    };

    // name (text)
    let name = match &record.values[1] {
        Value::Text(s) => s.clone(),
        _ => return None,
    };

    // tbl_name (text)
    let table_name = match &record.values[2] {
        Value::Text(s) => s.clone(),
        _ => return None,
    };

    // rootpage (integer)
    let root_page = match &record.values[3] {
        Value::Integer(i) => *i as u32,
        Value::Null => 0, // Views and triggers have NULL rootpage
        _ => return None,
    };

    // sql (text or null)
    let sql = match &record.values[4] {
        Value::Text(s) => Some(s.clone()),
        Value::Null => None,
        _ => None,
    };

    Some(SchemaEntry {
        object_type,
        name,
        table_name,
        root_page,
        sql,
    })
}

/// Recursively collect schema entries from a potentially multi-page sqlite_schema
pub fn collect_schema_entries<F>(
    root_page: &Page,
    mut read_page: F,
    usable_size: u32,
) -> Result<Vec<SchemaEntry>>
where
    F: FnMut(u32) -> Result<Page>,
{
    let mut entries = Vec::new();

    match root_page.page_type {
        PageType::LeafTable => {
            // Leaf page - extract records directly
            for cell in &root_page.cells {
                if let Cell::TableLeaf(leaf_cell) = cell {
                    if let Some(record) = &leaf_cell.payload {
                        if let Some(entry) = parse_schema_record(record) {
                            entries.push(entry);
                        }
                    }
                }
            }
        }
        PageType::InteriorTable => {
            // Interior page - traverse children
            for cell in &root_page.cells {
                if let Cell::TableInterior(interior_cell) = cell {
                    let child_page = read_page(interior_cell.left_child_page)?;
                    let child_entries = collect_schema_entries(&child_page, &mut read_page, usable_size)?;
                    entries.extend(child_entries);
                }
            }
            // Don't forget the rightmost pointer
            if let Some(header) = &root_page.header {
                if let Some(right_page) = header.right_most_pointer {
                    let child_page = read_page(right_page)?;
                    let child_entries = collect_schema_entries(&child_page, &mut read_page, usable_size)?;
                    entries.extend(child_entries);
                }
            }
        }
        _ => {
            return Err(SqliteVizError::SchemaError(
                "Unexpected page type in sqlite_schema".to_string()
            ));
        }
    }

    Ok(entries)
}
