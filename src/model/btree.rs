use serde::Serialize;
use crate::model::PageType;

/// Type of B-tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BTreeType {
    Table,
    Index,
}

/// A complete B-tree structure
#[derive(Debug, Clone, Serialize)]
pub struct BTree {
    /// Name of the table or index
    pub name: String,
    /// Root page number
    pub root_page: u32,
    /// Type of B-tree
    pub tree_type: BTreeType,
    /// All nodes in the tree
    pub nodes: Vec<BTreeNode>,
    /// Maximum depth of the tree
    pub depth: usize,
    /// Total number of cells across all pages
    pub total_cells: usize,
}

/// A node in the B-tree (corresponds to a page)
#[derive(Debug, Clone, Serialize)]
pub struct BTreeNode {
    /// Page number
    pub page_number: u32,
    /// Page type
    pub page_type: PageType,
    /// Depth in tree (0 = root)
    pub depth: usize,
    /// Number of cells in this page
    pub cell_count: usize,
    /// Child page numbers (from cell pointers + rightmost pointer)
    pub children: Vec<u32>,
    /// Overflow chains originating from this page
    pub overflow_chains: Vec<OverflowChain>,
    /// Bytes used in page
    pub size_used: usize,
    /// Free bytes in page
    pub size_free: usize,
    /// Parent page number (None for root)
    pub parent: Option<u32>,
}

/// A chain of overflow pages
#[derive(Debug, Clone, Serialize)]
pub struct OverflowChain {
    /// Cell index that this overflow belongs to
    pub cell_index: usize,
    /// Page numbers in the chain
    pub pages: Vec<u32>,
    /// Total bytes in overflow
    pub total_bytes: usize,
}
