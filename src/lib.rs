pub mod error;
pub mod parser;
pub mod model;
pub mod analyzer;
pub mod visualization;
pub mod dump;

use std::path::Path;
use memmap2::Mmap;
use std::fs::File;

use crate::error::{Result, SqliteVizError};
use crate::model::{DatabaseHeader, Page, Schema, BTree, BTreeType};
use crate::parser::{parse_database_header, parse_page};
use crate::analyzer::{parse_schema, build_btree};
use crate::visualization::{VizData, VizDatabaseInfo, VizSchema, VizBTree, VizPage, generate_html};

/// Main database reader
pub struct Database {
    mmap: Mmap,
    pub header: DatabaseHeader,
    file_name: String,
}

impl Database {
    /// Open a SQLite database file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < 100 {
            return Err(SqliteVizError::UnexpectedEof { context: "database file" });
        }

        let header = parse_database_header(&mmap[..100])?;
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("database")
            .to_string();

        Ok(Self { mmap, header, file_name })
    }

    /// Get the number of pages in the database
    pub fn page_count(&self) -> u32 {
        if self.header.database_size_pages > 0 {
            self.header.database_size_pages
        } else {
            // Calculate from file size
            (self.mmap.len() / self.header.page_size as usize) as u32
        }
    }

    /// Read raw page data
    pub fn read_page_raw(&self, page_number: u32) -> Result<&[u8]> {
        if page_number < 1 || page_number > self.page_count() {
            return Err(SqliteVizError::PageOutOfBounds {
                page: page_number,
                total: self.page_count(),
            });
        }

        let page_size = self.header.page_size as usize;
        let offset = (page_number as usize - 1) * page_size;
        let end = offset + page_size;

        if end > self.mmap.len() {
            return Err(SqliteVizError::PageOutOfBounds {
                page: page_number,
                total: self.page_count(),
            });
        }

        Ok(&self.mmap[offset..end])
    }

    /// Parse a page
    pub fn parse_page(&self, page_number: u32) -> Result<Page> {
        let page_data = self.read_page_raw(page_number)?;
        parse_page(
            page_data,
            page_number,
            self.header.page_size,
            self.header.usable_size(),
        )
    }

    /// Parse the database schema
    pub fn parse_schema(&self) -> Result<Schema> {
        let page1 = self.parse_page(1)?;
        parse_schema(&page1)
    }

    /// Build a B-tree for a table or index
    pub fn build_btree(&self, name: &str, root_page: u32, tree_type: BTreeType) -> Result<BTree> {
        build_btree(
            name.to_string(),
            root_page,
            tree_type,
            |page_num| self.parse_page(page_num),
            self.header.usable_size(),
        )
    }

    /// Generate visualization data for the entire database
    pub fn generate_viz_data(&self, filter_tables: Option<&[String]>, filter_indexes: Option<&[String]>) -> Result<VizData> {
        let schema = self.parse_schema()?;

        // Build B-trees for tables and indexes
        let mut btrees = Vec::new();
        let mut all_pages = Vec::new();
        let mut seen_pages = std::collections::HashSet::new();

        // sqlite_schema B-tree (always included)
        let schema_btree = self.build_btree("sqlite_schema", 1, BTreeType::Table)?;
        for node in &schema_btree.nodes {
            if seen_pages.insert(node.page_number) {
                all_pages.push(self.parse_page(node.page_number)?);
            }
        }
        btrees.push(schema_btree);

        // Tables
        for entry in schema.tables() {
            // Skip internal tables and apply filter
            if entry.name.starts_with("sqlite_") {
                continue;
            }
            if let Some(filter) = filter_tables {
                if !filter.contains(&entry.name) {
                    continue;
                }
            }
            if entry.root_page == 0 {
                continue;
            }

            let btree = self.build_btree(&entry.name, entry.root_page, BTreeType::Table)?;
            for node in &btree.nodes {
                if seen_pages.insert(node.page_number) {
                    all_pages.push(self.parse_page(node.page_number)?);
                }
            }
            btrees.push(btree);
        }

        // Indexes
        for entry in schema.indexes() {
            // Apply filter
            if let Some(filter) = filter_indexes {
                if !filter.contains(&entry.name) {
                    continue;
                }
            }
            if entry.root_page == 0 {
                continue;
            }

            let btree = self.build_btree(&entry.name, entry.root_page, BTreeType::Index)?;
            for node in &btree.nodes {
                if seen_pages.insert(node.page_number) {
                    all_pages.push(self.parse_page(node.page_number)?);
                }
            }
            btrees.push(btree);
        }

        Ok(VizData {
            database_info: VizDatabaseInfo::from_header(&self.header, self.file_name.clone()),
            schema: VizSchema::from_schema(&schema),
            btrees: btrees.iter().map(VizBTree::from_btree).collect(),
            pages: all_pages.iter().map(VizPage::from_page).collect(),
        })
    }

    /// Generate HTML visualization file
    pub fn generate_visualization<P: AsRef<Path>>(
        &self,
        output_path: P,
        filter_tables: Option<&[String]>,
        filter_indexes: Option<&[String]>,
    ) -> Result<()> {
        let viz_data = self.generate_viz_data(filter_tables, filter_indexes)?;
        generate_html(&viz_data, output_path.as_ref())
    }
}

/// Print database info
pub fn print_database_info(db: &Database, verbose: bool) {
    let header = &db.header;

    println!("SQLite Database Information");
    println!("===========================");
    println!("File: {}", db.file_name);
    println!("Page size: {} bytes", header.page_size);
    println!("Page count: {}", db.page_count());
    println!("SQLite version: {}", header.sqlite_version_string());
    println!("Text encoding: {:?}", header.text_encoding);
    println!("Schema format: {}", header.schema_format);
    println!("User version: {}", header.user_version);

    if header.first_freelist_page > 0 {
        println!("Freelist pages: {} (first: {})", header.freelist_page_count, header.first_freelist_page);
    }

    if verbose {
        println!();
        println!("Schema");
        println!("------");

        if let Ok(schema) = db.parse_schema() {
            for entry in schema.tables() {
                println!("  TABLE {} (root page {})", entry.name, entry.root_page);
            }
            for entry in schema.indexes() {
                println!("  INDEX {} on {} (root page {})", entry.name, entry.table_name, entry.root_page);
            }
        }

        println!();
        println!("B-tree Statistics");
        println!("-----------------");

        if let Ok(schema) = db.parse_schema() {
            for entry in schema.tables() {
                if entry.root_page == 0 || entry.name.starts_with("sqlite_") {
                    continue;
                }
                if let Ok(btree) = db.build_btree(&entry.name, entry.root_page, BTreeType::Table) {
                    let stats = btree.stats();
                    println!("  {} (table):", entry.name);
                    println!("    Pages: {} (interior: {}, leaf: {})", stats.total_pages, stats.interior_pages, stats.leaf_pages);
                    println!("    Cells: {}, Depth: {}", stats.total_cells, stats.depth);
                    if stats.overflow_pages > 0 {
                        println!("    Overflow pages: {}", stats.overflow_pages);
                    }
                }
            }
        }
    }
}
