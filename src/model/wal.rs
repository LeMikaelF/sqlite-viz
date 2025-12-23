//! WAL (Write-Ahead Log) data structures.

use serde::Serialize;

use super::Page;

/// WAL file magic number for big-endian checksums
pub const WAL_MAGIC_BIG_ENDIAN: u32 = 0x377f0682;
/// WAL file magic number for little-endian checksums
pub const WAL_MAGIC_LITTLE_ENDIAN: u32 = 0x377f0683;

/// WAL file header (32 bytes)
#[derive(Debug, Clone, Serialize)]
pub struct WalHeader {
    /// Magic number (0x377f0682 or 0x377f0683)
    pub magic: u32,
    /// File format version (currently 3007000)
    pub format_version: u32,
    /// Database page size
    pub page_size: u32,
    /// Checkpoint sequence number
    pub checkpoint_sequence: u32,
    /// Salt-1: random integer incremented with each checkpoint
    pub salt1: u32,
    /// Salt-2: different random integer per checkpoint
    pub salt2: u32,
    /// Checksum-1: cumulative checksum over header
    pub checksum1: u32,
    /// Checksum-2: cumulative checksum over header
    pub checksum2: u32,
}

impl WalHeader {
    /// Check if this WAL uses big-endian checksums
    pub fn is_big_endian(&self) -> bool {
        self.magic == WAL_MAGIC_BIG_ENDIAN
    }
}

/// WAL frame header (24 bytes)
#[derive(Debug, Clone, Serialize)]
pub struct WalFrameHeader {
    /// Page number in the database file
    pub page_number: u32,
    /// Size of database in pages after this commit (0 if not a commit frame)
    pub db_size_after_commit: u32,
    /// Salt-1 copy (must match WAL header)
    pub salt1: u32,
    /// Salt-2 copy (must match WAL header)
    pub salt2: u32,
    /// Checksum-1 for this frame
    pub checksum1: u32,
    /// Checksum-2 for this frame
    pub checksum2: u32,
}

impl WalFrameHeader {
    /// Returns true if this frame is a commit frame
    pub fn is_commit_frame(&self) -> bool {
        self.db_size_after_commit > 0
    }
}

/// A complete WAL frame (header + page data)
#[derive(Debug, Clone)]
pub struct WalFrame {
    /// Frame index (0-based position in WAL file)
    pub frame_index: usize,
    /// Frame header
    pub header: WalFrameHeader,
    /// Parsed page content (None if page could not be parsed)
    pub page: Option<Page>,
    /// Raw page data
    pub raw_page_data: Vec<u8>,
}

/// Parsed WAL file structure
#[derive(Debug, Clone)]
pub struct WalFile {
    /// WAL header
    pub header: WalHeader,
    /// All frames in the WAL
    pub frames: Vec<WalFrame>,
    /// Source file name
    pub file_name: String,
}
