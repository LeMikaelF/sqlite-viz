use serde::Serialize;
use crate::model::{BTree, DatabaseHeader, Schema, Page, Cell};

/// Root visualization data structure
#[derive(Debug, Serialize)]
pub struct VizData {
    pub database_info: VizDatabaseInfo,
    pub schema: VizSchema,
    pub btrees: Vec<VizBTree>,
    pub pages: Vec<VizPage>,
}

#[derive(Debug, Serialize)]
pub struct VizDatabaseInfo {
    pub file_name: String,
    pub page_size: u32,
    pub page_count: u32,
    pub usable_size: u32,
    pub text_encoding: String,
    pub sqlite_version: String,
    pub schema_format: u32,
}

impl VizDatabaseInfo {
    pub fn from_header(header: &DatabaseHeader, file_name: String) -> Self {
        Self {
            file_name,
            page_size: header.page_size,
            page_count: header.database_size_pages,
            usable_size: header.usable_size(),
            text_encoding: format!("{:?}", header.text_encoding),
            sqlite_version: header.sqlite_version_string(),
            schema_format: header.schema_format,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VizSchema {
    pub tables: Vec<VizSchemaEntry>,
    pub indexes: Vec<VizSchemaEntry>,
}

#[derive(Debug, Serialize)]
pub struct VizSchemaEntry {
    pub name: String,
    pub table_name: String,
    pub root_page: u32,
    pub sql: Option<String>,
}

impl VizSchema {
    pub fn from_schema(schema: &Schema) -> Self {
        let tables: Vec<_> = schema.tables()
            .map(|e| VizSchemaEntry {
                name: e.name.clone(),
                table_name: e.table_name.clone(),
                root_page: e.root_page,
                sql: e.sql.clone(),
            })
            .collect();

        let indexes: Vec<_> = schema.indexes()
            .map(|e| VizSchemaEntry {
                name: e.name.clone(),
                table_name: e.table_name.clone(),
                root_page: e.root_page,
                sql: e.sql.clone(),
            })
            .collect();

        Self { tables, indexes }
    }
}

#[derive(Debug, Serialize)]
pub struct VizBTree {
    pub name: String,
    pub tree_type: String,
    pub root_page: u32,
    pub depth: usize,
    pub total_cells: usize,
    pub total_pages: usize,
    pub nodes: Vec<VizBTreeNode>,
    pub links: Vec<VizLink>,
}

#[derive(Debug, Serialize)]
pub struct VizBTreeNode {
    pub id: String,
    pub page_number: u32,
    pub page_type: String,
    pub depth: usize,
    pub cell_count: usize,
    pub children: Vec<u32>,
    pub size_used: usize,
    pub size_free: usize,
    pub has_overflow: bool,
    pub overflow_pages: Vec<u32>,
}

#[derive(Debug, Serialize)]
pub struct VizLink {
    pub source: String,
    pub target: String,
    pub link_type: String,
}

impl VizBTree {
    pub fn from_btree(btree: &BTree) -> Self {
        let nodes: Vec<_> = btree.nodes.iter().map(|node| {
            let overflow_pages: Vec<_> = node.overflow_chains.iter()
                .flat_map(|c| &c.pages)
                .copied()
                .collect();

            VizBTreeNode {
                id: format!("p{}", node.page_number),
                page_number: node.page_number,
                page_type: format!("{:?}", node.page_type),
                depth: node.depth,
                cell_count: node.cell_count,
                children: node.children.clone(),
                size_used: node.size_used,
                size_free: node.size_free,
                has_overflow: !node.overflow_chains.is_empty(),
                overflow_pages,
            }
        }).collect();

        // Build links
        let mut links = Vec::new();

        for node in &btree.nodes {
            let source_id = format!("p{}", node.page_number);

            // Child links
            for &child in &node.children {
                links.push(VizLink {
                    source: source_id.clone(),
                    target: format!("p{}", child),
                    link_type: "child".to_string(),
                });
            }

            // Overflow links
            for chain in &node.overflow_chains {
                for &overflow_page in &chain.pages {
                    links.push(VizLink {
                        source: source_id.clone(),
                        target: format!("o{}", overflow_page),
                        link_type: "overflow".to_string(),
                    });
                }
            }
        }

        Self {
            name: btree.name.clone(),
            tree_type: format!("{:?}", btree.tree_type),
            root_page: btree.root_page,
            depth: btree.depth,
            total_cells: btree.total_cells,
            total_pages: btree.nodes.len(),
            nodes,
            links,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VizPage {
    pub page_number: u32,
    pub page_type: String,
    pub cell_count: usize,
    pub free_space: usize,
    pub cell_content_start: u16,
    pub cells: Vec<VizCell>,
}

#[derive(Debug, Serialize)]
pub struct VizCell {
    pub index: usize,
    pub offset: u16,
    pub size: usize,
    pub cell_type: String,
    pub rowid: Option<i64>,
    pub left_child: Option<u32>,
    pub payload_size: Option<u64>,
    pub has_overflow: bool,
    pub overflow_page: Option<u32>,
    pub preview: String,
}

impl VizPage {
    pub fn from_page(page: &Page) -> Self {
        let cells: Vec<_> = page.cells.iter().enumerate().map(|(i, cell)| {
            let preview = match cell {
                Cell::TableLeaf(c) => {
                    if let Some(record) = &c.payload {
                        record.values.iter()
                            .take(3)
                            .map(|v| v.preview(20))
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        format!("rowid={}", c.rowid)
                    }
                }
                Cell::TableInterior(c) => format!("rowid={}, child={}", c.rowid, c.left_child_page),
                Cell::IndexLeaf(c) => {
                    if let Some(record) = &c.payload {
                        record.values.iter()
                            .take(3)
                            .map(|v| v.preview(20))
                            .collect::<Vec<_>>()
                            .join(", ")
                    } else {
                        format!("payload_size={}", c.payload_size)
                    }
                }
                Cell::IndexInterior(c) => {
                    format!("child={}, payload_size={}", c.left_child_page, c.payload_size)
                }
            };

            VizCell {
                index: i,
                offset: cell.cell_offset(),
                size: cell.cell_size(),
                cell_type: match cell {
                    Cell::TableLeaf(_) => "TableLeaf",
                    Cell::TableInterior(_) => "TableInterior",
                    Cell::IndexLeaf(_) => "IndexLeaf",
                    Cell::IndexInterior(_) => "IndexInterior",
                }.to_string(),
                rowid: cell.rowid(),
                left_child: cell.left_child(),
                payload_size: cell.payload_size(),
                has_overflow: cell.overflow_page().is_some(),
                overflow_page: cell.overflow_page(),
                preview,
            }
        }).collect();

        let cell_content_start = page.header
            .as_ref()
            .map(|h| h.cell_content_start)
            .unwrap_or(0);

        Self {
            page_number: page.page_number,
            page_type: format!("{:?}", page.page_type),
            cell_count: page.cells.len(),
            free_space: page.free_space,
            cell_content_start,
            cells,
        }
    }
}
