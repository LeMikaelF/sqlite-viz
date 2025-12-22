use crate::error::{Result, SqliteVizError};
use crate::model::OverflowPage;

/// Information about an overflow chain
#[derive(Debug, Clone)]
pub struct OverflowChainInfo {
    /// All pages in the chain
    pub pages: Vec<OverflowPage>,
    /// Total payload bytes across all overflow pages
    pub total_bytes: usize,
}

/// Parse an overflow page header to get next page and content info
pub fn parse_overflow_header(page_data: &[u8]) -> Result<(Option<u32>, usize)> {
    if page_data.len() < 4 {
        return Err(SqliteVizError::UnexpectedEof { context: "overflow page header" });
    }

    let next_page = u32::from_be_bytes([page_data[0], page_data[1], page_data[2], page_data[3]]);
    let next_page = if next_page == 0 { None } else { Some(next_page) };

    // Content starts at byte 4
    let content_size = page_data.len() - 4;

    Ok((next_page, content_size))
}

/// Follow an overflow chain and collect all pages
pub fn follow_overflow_chain<F>(
    first_overflow_page: u32,
    usable_size: u32,
    remaining_bytes: usize,
    mut read_page: F,
) -> Result<OverflowChainInfo>
where
    F: FnMut(u32) -> Result<Vec<u8>>,
{
    let mut pages = Vec::new();
    let mut current_page = Some(first_overflow_page);
    let mut bytes_remaining = remaining_bytes;
    let mut total_bytes = 0;

    // Maximum content per overflow page
    let max_content_per_page = usable_size as usize - 4;

    while let Some(page_num) = current_page {
        let page_data = read_page(page_num)?;
        let (next_page, _) = parse_overflow_header(&page_data)?;

        // Calculate content size for this page
        let content_size = bytes_remaining.min(max_content_per_page);
        bytes_remaining = bytes_remaining.saturating_sub(content_size);
        total_bytes += content_size;

        pages.push(OverflowPage {
            page_number: page_num,
            next_page,
            content_size,
        });

        current_page = next_page;

        // Safety: prevent infinite loops
        if pages.len() > 100000 {
            break;
        }
    }

    Ok(OverflowChainInfo { pages, total_bytes })
}
