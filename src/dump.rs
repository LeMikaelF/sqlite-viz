//! Human-readable text dump functionality for debugging SQLite databases.

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::Result;
use crate::model::{
    BTree, BTreeNode, BTreeType, Cell, DatabaseHeader, Page, PageType, Record, SerialType, Value,
};
use crate::Database;

/// Options for controlling what gets dumped
pub struct DumpOptions {
    /// Specific B-trees to dump (by name). If None, dumps all.
    pub btrees: Option<Vec<String>>,
    /// Specific pages to dump (by number). If None, dumps based on btrees.
    pub pages: Option<Vec<u32>>,
}

/// Dump database information to a file
pub fn dump_to_file(db: &Database, output_path: &Path, options: &DumpOptions) -> Result<()> {
    let content = dump_to_string(db, options)?;
    let mut file = File::create(output_path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Dump database information to a string
pub fn dump_to_string(db: &Database, options: &DumpOptions) -> Result<String> {
    let mut out = String::new();

    // Header
    writeln!(out, "================================================================================").unwrap();
    writeln!(out, "SQLite Database Dump").unwrap();
    writeln!(out, "================================================================================").unwrap();
    writeln!(out).unwrap();

    // Database header info
    dump_header(&mut out, &db.header, db.page_count());

    // If specific pages requested, just dump those
    if let Some(page_numbers) = &options.pages {
        writeln!(out).unwrap();
        writeln!(out, "================================================================================").unwrap();
        writeln!(out, "REQUESTED PAGES").unwrap();
        writeln!(out, "================================================================================").unwrap();

        for &page_num in page_numbers {
            writeln!(out).unwrap();
            match db.parse_page(page_num) {
                Ok(page) => {
                    let raw_data = db.read_page_raw(page_num).ok();
                    dump_page(&mut out, &page, raw_data);
                }
                Err(e) => {
                    writeln!(out, "ERROR: Could not parse page {}: {}", page_num, e).unwrap();
                }
            }
        }
        return Ok(out);
    }

    // Parse schema
    let schema = db.parse_schema()?;

    // Dump schema overview
    writeln!(out).unwrap();
    writeln!(out, "================================================================================").unwrap();
    writeln!(out, "SCHEMA").unwrap();
    writeln!(out, "================================================================================").unwrap();
    writeln!(out).unwrap();

    for entry in schema.tables() {
        writeln!(out, "TABLE: {} (root page {})", entry.name, entry.root_page).unwrap();
        if let Some(sql) = &entry.sql {
            writeln!(out, "  SQL: {}", sql.replace('\n', "\n       ")).unwrap();
        }
    }
    writeln!(out).unwrap();
    for entry in schema.indexes() {
        writeln!(out, "INDEX: {} on {} (root page {})", entry.name, entry.table_name, entry.root_page).unwrap();
        if let Some(sql) = &entry.sql {
            writeln!(out, "  SQL: {}", sql.replace('\n', "\n       ")).unwrap();
        }
    }

    // Determine which B-trees to dump
    let mut btrees_to_dump: Vec<(String, u32, BTreeType)> = Vec::new();

    // Always include sqlite_schema
    let include_all = options.btrees.is_none();
    let filter_names = options.btrees.as_ref();

    if include_all || filter_names.map_or(false, |f| f.iter().any(|n| n == "sqlite_schema")) {
        btrees_to_dump.push(("sqlite_schema".to_string(), 1, BTreeType::Table));
    }

    for entry in schema.tables() {
        if entry.name.starts_with("sqlite_") || entry.root_page == 0 {
            continue;
        }
        if include_all || filter_names.map_or(false, |f| f.contains(&entry.name)) {
            btrees_to_dump.push((entry.name.clone(), entry.root_page, BTreeType::Table));
        }
    }

    for entry in schema.indexes() {
        if entry.root_page == 0 {
            continue;
        }
        if include_all || filter_names.map_or(false, |f| f.contains(&entry.name)) {
            btrees_to_dump.push((entry.name.clone(), entry.root_page, BTreeType::Index));
        }
    }

    // Dump each B-tree
    for (name, root_page, tree_type) in btrees_to_dump {
        writeln!(out).unwrap();
        writeln!(out, "================================================================================").unwrap();
        writeln!(out, "B-TREE: {} ({:?})", name, tree_type).unwrap();
        writeln!(out, "================================================================================").unwrap();

        match db.build_btree(&name, root_page, tree_type) {
            Ok(btree) => {
                dump_btree(&mut out, db, &btree)?;
            }
            Err(e) => {
                writeln!(out, "ERROR: Could not build B-tree: {}", e).unwrap();
            }
        }
    }

    Ok(out)
}

fn dump_header(out: &mut String, header: &DatabaseHeader, page_count: u32) {
    writeln!(out, "DATABASE HEADER").unwrap();
    writeln!(out, "--------------------------------------------------------------------------------").unwrap();
    writeln!(out, "Page size:              {} bytes", header.page_size).unwrap();
    writeln!(out, "Usable size:            {} bytes", header.usable_size()).unwrap();
    writeln!(out, "Page count:             {}", page_count).unwrap();
    writeln!(out, "File format (r/w):      {}/{}", header.file_format_read, header.file_format_write).unwrap();
    writeln!(out, "Reserved bytes/page:    {}", header.reserved_bytes_per_page).unwrap();
    writeln!(out, "Text encoding:          {:?}", header.text_encoding).unwrap();
    writeln!(out, "Schema format:          {}", header.schema_format).unwrap();
    writeln!(out, "Schema cookie:          {}", header.schema_cookie).unwrap();
    writeln!(out, "User version:           {}", header.user_version).unwrap();
    writeln!(out, "Application ID:         {}", header.application_id).unwrap();
    writeln!(out, "SQLite version:         {}", header.sqlite_version_string()).unwrap();
    writeln!(out, "File change counter:    {}", header.file_change_counter).unwrap();
    writeln!(out, "First freelist page:    {}", header.first_freelist_page).unwrap();
    writeln!(out, "Freelist page count:    {}", header.freelist_page_count).unwrap();
    writeln!(out, "Max payload fraction:   {}", header.max_payload_fraction).unwrap();
    writeln!(out, "Min payload fraction:   {}", header.min_payload_fraction).unwrap();
    writeln!(out, "Leaf payload fraction:  {}", header.leaf_payload_fraction).unwrap();
    writeln!(out, "Largest root B-tree:    {}", header.largest_root_page).unwrap();
    writeln!(out, "Incremental vacuum:     {}", header.incremental_vacuum).unwrap();
    writeln!(out, "Version valid for:      {}", header.version_valid_for).unwrap();
}

fn dump_btree(out: &mut String, db: &Database, btree: &BTree) -> Result<()> {
    writeln!(out).unwrap();
    writeln!(out, "Root page:     {}", btree.root_page).unwrap();
    writeln!(out, "Tree depth:    {}", btree.depth).unwrap();
    writeln!(out, "Total nodes:   {}", btree.nodes.len()).unwrap();
    writeln!(out, "Total cells:   {}", btree.total_cells).unwrap();

    // Print tree structure overview
    writeln!(out).unwrap();
    writeln!(out, "Tree Structure:").unwrap();
    dump_tree_structure(out, btree);

    // Dump each page in the tree
    for node in &btree.nodes {
        writeln!(out).unwrap();
        writeln!(out, "--------------------------------------------------------------------------------").unwrap();

        let page = db.parse_page(node.page_number)?;
        let raw_data = db.read_page_raw(node.page_number).ok();
        dump_page_with_node(out, &page, node, raw_data);
    }

    Ok(())
}

fn dump_tree_structure(out: &mut String, btree: &BTree) {
    // Group nodes by depth
    let mut by_depth: Vec<Vec<&BTreeNode>> = vec![Vec::new(); btree.depth + 1];
    for node in &btree.nodes {
        if node.depth <= btree.depth {
            by_depth[node.depth].push(node);
        }
    }

    for (depth, nodes) in by_depth.iter().enumerate() {
        let indent = "  ".repeat(depth);
        let level_type = if depth == btree.depth { "leaf" } else { "interior" };
        write!(out, "{}Level {} ({}):", indent, depth, level_type).unwrap();

        for node in nodes {
            write!(out, " [p{}:{}c]", node.page_number, node.cell_count).unwrap();
        }
        writeln!(out).unwrap();
    }
}

fn dump_page_with_node(out: &mut String, page: &Page, node: &BTreeNode, raw_data: Option<&[u8]>) {
    writeln!(out, "PAGE {} (depth {}, {:?})", page.page_number, node.depth, page.page_type).unwrap();

    if let Some(parent) = node.parent {
        writeln!(out, "  Parent page: {}", parent).unwrap();
    }

    dump_page_common(out, page, raw_data);

    // Overflow info
    if !node.overflow_chains.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "  Overflow chains:").unwrap();
        for chain in &node.overflow_chains {
            writeln!(
                out,
                "    Cell {}: {} pages, {} bytes → {:?}",
                chain.cell_index, chain.pages.len(), chain.total_bytes, chain.pages
            ).unwrap();
        }
    }
}

fn dump_page(out: &mut String, page: &Page, raw_data: Option<&[u8]>) {
    writeln!(out, "PAGE {} ({:?})", page.page_number, page.page_type).unwrap();
    dump_page_common(out, page, raw_data);
}

fn dump_page_common(out: &mut String, page: &Page, raw_data: Option<&[u8]>) {
    // Header info
    if let Some(header) = &page.header {
        writeln!(out, "  Header:").unwrap();
        writeln!(out, "    Page type byte:       0x{:02x}", match header.page_type {
            PageType::InteriorIndex => 0x02,
            PageType::InteriorTable => 0x05,
            PageType::LeafIndex => 0x0a,
            PageType::LeafTable => 0x0d,
            _ => 0x00,
        }).unwrap();
        writeln!(out, "    First freeblock:      {}", header.first_freeblock).unwrap();
        writeln!(out, "    Cell count:           {}", header.cell_count).unwrap();
        writeln!(out, "    Cell content start:   {}", header.cell_content_start).unwrap();
        writeln!(out, "    Fragmented bytes:     {}", header.fragmented_free_bytes).unwrap();
        if let Some(rmp) = header.right_most_pointer {
            writeln!(out, "    Right-most pointer:   {}", rmp).unwrap();
        }
    }

    writeln!(out, "  Free space:             {} bytes", page.free_space).unwrap();

    // Cell pointers
    if !page.cell_pointers.is_empty() {
        writeln!(out, "  Cell pointers:          {:?}", page.cell_pointers).unwrap();
    }

    // Cells
    writeln!(out).unwrap();
    writeln!(out, "  Cells ({}):", page.cells.len()).unwrap();

    for (i, cell) in page.cells.iter().enumerate() {
        dump_cell(out, i, cell);
    }

    // Hex dump
    if let Some(data) = raw_data {
        writeln!(out).unwrap();
        writeln!(out, "  Hex dump:").unwrap();
        dump_hex(out, data, "    ");
    }
}

fn dump_cell(out: &mut String, index: usize, cell: &Cell) {
    match cell {
        Cell::TableLeaf(c) => {
            writeln!(out, "    [{}] TableLeafCell @ offset {}, {} bytes", index, c.cell_offset, c.cell_size).unwrap();
            writeln!(out, "        rowid: {}", c.rowid).unwrap();
            writeln!(out, "        payload size: {} (local: {})", c.payload_size, c.local_payload_size).unwrap();
            if let Some(overflow) = c.overflow_page {
                writeln!(out, "        overflow page: {}", overflow).unwrap();
            }
            if let Some(record) = &c.payload {
                dump_record(out, record, "        ");
            }
        }
        Cell::TableInterior(c) => {
            writeln!(out, "    [{}] TableInteriorCell @ offset {}, {} bytes", index, c.cell_offset, c.cell_size).unwrap();
            writeln!(out, "        left child: page {}", c.left_child_page).unwrap();
            writeln!(out, "        rowid: {}", c.rowid).unwrap();
        }
        Cell::IndexLeaf(c) => {
            writeln!(out, "    [{}] IndexLeafCell @ offset {}, {} bytes", index, c.cell_offset, c.cell_size).unwrap();
            writeln!(out, "        payload size: {} (local: {})", c.payload_size, c.local_payload_size).unwrap();
            if let Some(overflow) = c.overflow_page {
                writeln!(out, "        overflow page: {}", overflow).unwrap();
            }
            if let Some(record) = &c.payload {
                dump_record(out, record, "        ");
            }
        }
        Cell::IndexInterior(c) => {
            writeln!(out, "    [{}] IndexInteriorCell @ offset {}, {} bytes", index, c.cell_offset, c.cell_size).unwrap();
            writeln!(out, "        left child: page {}", c.left_child_page).unwrap();
            writeln!(out, "        payload size: {} (local: {})", c.payload_size, c.local_payload_size).unwrap();
            if let Some(overflow) = c.overflow_page {
                writeln!(out, "        overflow page: {}", overflow).unwrap();
            }
            if let Some(record) = &c.payload {
                dump_record(out, record, "        ");
            }
        }
    }
}

fn dump_record(out: &mut String, record: &Record, indent: &str) {
    writeln!(out, "{}record header size: {}", indent, record.header_size).unwrap();
    writeln!(out, "{}columns ({}):", indent, record.values.len()).unwrap();

    for (i, (serial_type, value)) in record.column_types.iter().zip(record.values.iter()).enumerate() {
        let type_str = format_serial_type(serial_type);
        let value_str = format_value(value);
        writeln!(out, "{}  [{}] {} = {}", indent, i, type_str, value_str).unwrap();
    }
}

fn format_serial_type(st: &SerialType) -> String {
    match st {
        SerialType::Null => "NULL".to_string(),
        SerialType::Int8 => "Int8".to_string(),
        SerialType::Int16 => "Int16".to_string(),
        SerialType::Int24 => "Int24".to_string(),
        SerialType::Int32 => "Int32".to_string(),
        SerialType::Int48 => "Int48".to_string(),
        SerialType::Int64 => "Int64".to_string(),
        SerialType::Float64 => "Float64".to_string(),
        SerialType::Zero => "Zero".to_string(),
        SerialType::One => "One".to_string(),
        SerialType::Reserved(n) => format!("Reserved({})", n),
        SerialType::Blob(n) => format!("Blob({})", n),
        SerialType::Text(n) => format!("Text({})", n),
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => format!("{}", f),
        Value::Text(s) => {
            // Truncate long text and escape special characters
            let display = if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.clone()
            };
            format!("\"{}\"", display.replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t"))
        }
        Value::Blob(b) => {
            if b.len() <= 32 {
                format!("x'{}'", hex_encode(b))
            } else {
                format!("x'{}...' ({} bytes)", hex_encode(&b[..32]), b.len())
            }
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn dump_hex(out: &mut String, data: &[u8], indent: &str) {
    for (offset, chunk) in data.chunks(16).enumerate() {
        let offset = offset * 16;
        write!(out, "{}{:08x}  ", indent, offset).unwrap();

        // Hex bytes
        for (i, byte) in chunk.iter().enumerate() {
            if i == 8 {
                write!(out, " ").unwrap();
            }
            write!(out, "{:02x} ", byte).unwrap();
        }

        // Padding for incomplete lines
        for i in chunk.len()..16 {
            if i == 8 {
                write!(out, " ").unwrap();
            }
            write!(out, "   ").unwrap();
        }

        // ASCII representation
        write!(out, " |").unwrap();
        for byte in chunk {
            let c = if *byte >= 0x20 && *byte < 0x7f {
                *byte as char
            } else {
                '.'
            };
            write!(out, "{}", c).unwrap();
        }
        writeln!(out, "|").unwrap();
    }
}
