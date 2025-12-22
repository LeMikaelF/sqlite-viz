use crate::error::Result;
use crate::model::{BTree, BTreeNode, BTreeType, OverflowChain, Page, Cell};
use crate::parser::overflow::follow_overflow_chain;

/// Build a complete B-tree structure by traversing from the root page
pub fn build_btree<F>(
    name: String,
    root_page_num: u32,
    tree_type: BTreeType,
    mut read_page: F,
    _usable_size: u32,
) -> Result<BTree>
where
    F: FnMut(u32) -> Result<Page>,
{
    let mut nodes = Vec::new();
    let mut max_depth = 0;
    let mut total_cells = 0;

    // BFS traversal to build tree structure
    let mut queue: Vec<(u32, usize, Option<u32>)> = vec![(root_page_num, 0, None)];

    while let Some((page_num, depth, parent)) = queue.pop() {
        let page = read_page(page_num)?;
        max_depth = max_depth.max(depth);
        total_cells += page.cells.len();

        // Collect children and overflow chains
        let mut children = Vec::new();
        let mut overflow_chains = Vec::new();

        for (i, cell) in page.cells.iter().enumerate() {
            // Collect child pointers
            if let Some(child) = cell.left_child() {
                children.push(child);
            }

            // Track overflow chains
            if let Some(overflow_page) = cell.overflow_page() {
                if let Some(payload_size) = cell.payload_size() {
                    let local_size = match cell {
                        Cell::TableLeaf(c) => c.local_payload_size,
                        Cell::IndexLeaf(c) => c.local_payload_size,
                        Cell::IndexInterior(c) => c.local_payload_size,
                        _ => 0,
                    };
                    let overflow_bytes = payload_size as usize - local_size;

                    // We'll collect the pages later if needed
                    overflow_chains.push(OverflowChain {
                        cell_index: i,
                        pages: vec![overflow_page], // Just first page for now
                        total_bytes: overflow_bytes,
                    });
                }
            }
        }

        // Add rightmost pointer for interior pages
        if let Some(header) = &page.header {
            if let Some(right_ptr) = header.right_most_pointer {
                children.push(right_ptr);
            }
        }

        // Queue children for traversal
        for &child in &children {
            queue.push((child, depth + 1, Some(page_num)));
        }

        // Calculate space usage
        let header_size = if page.page_type.is_interior() { 12 } else { 8 };
        let page1_offset = if page_num == 1 { 100 } else { 0 };
        let cell_pointers_size = page.cells.len() * 2;
        let cells_size: usize = page.cells.iter().map(|c| c.cell_size()).sum();
        let size_used = page1_offset + header_size + cell_pointers_size + cells_size;
        let size_free = page.free_space;

        nodes.push(BTreeNode {
            page_number: page_num,
            page_type: page.page_type,
            depth,
            cell_count: page.cells.len(),
            children,
            overflow_chains,
            size_used,
            size_free,
            parent,
        });
    }

    // Sort nodes by page number for consistent output
    nodes.sort_by_key(|n| n.page_number);

    Ok(BTree {
        name,
        root_page: root_page_num,
        tree_type,
        nodes,
        depth: max_depth,
        total_cells,
    })
}

/// Expand overflow chains to include all pages in each chain
pub fn expand_overflow_chains<F>(
    btree: &mut BTree,
    mut read_page_raw: F,
    usable_size: u32,
) -> Result<()>
where
    F: FnMut(u32) -> Result<Vec<u8>>,
{
    for node in &mut btree.nodes {
        for chain in &mut node.overflow_chains {
            if !chain.pages.is_empty() {
                let first_page = chain.pages[0];
                let chain_info = follow_overflow_chain(
                    first_page,
                    usable_size,
                    chain.total_bytes,
                    &mut read_page_raw,
                )?;
                chain.pages = chain_info.pages.iter().map(|p| p.page_number).collect();
            }
        }
    }
    Ok(())
}

/// Get statistics about a B-tree
#[derive(Debug, Clone)]
pub struct BTreeStats {
    pub total_pages: usize,
    pub interior_pages: usize,
    pub leaf_pages: usize,
    pub total_cells: usize,
    pub overflow_pages: usize,
    pub depth: usize,
    pub total_size_used: usize,
    pub total_size_free: usize,
}

impl BTree {
    pub fn stats(&self) -> BTreeStats {
        let interior_pages = self.nodes.iter().filter(|n| n.page_type.is_interior()).count();
        let leaf_pages = self.nodes.iter().filter(|n| n.page_type.is_leaf()).count();
        let overflow_pages: usize = self.nodes.iter()
            .flat_map(|n| &n.overflow_chains)
            .map(|c| c.pages.len())
            .sum();
        let total_size_used: usize = self.nodes.iter().map(|n| n.size_used).sum();
        let total_size_free: usize = self.nodes.iter().map(|n| n.size_free).sum();

        BTreeStats {
            total_pages: self.nodes.len(),
            interior_pages,
            leaf_pages,
            total_cells: self.total_cells,
            overflow_pages,
            depth: self.depth,
            total_size_used,
            total_size_free,
        }
    }
}
