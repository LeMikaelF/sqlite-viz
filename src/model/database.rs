use serde::Serialize;

/// Text encoding used in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TextEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
}

impl TextEncoding {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(TextEncoding::Utf8),
            2 => Some(TextEncoding::Utf16Le),
            3 => Some(TextEncoding::Utf16Be),
            _ => None,
        }
    }
}

/// The 100-byte SQLite database header
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseHeader {
    /// Database page size in bytes (power of 2, 512-65536)
    pub page_size: u32,
    /// File format write version (1 = legacy, 2 = WAL)
    pub file_format_write: u8,
    /// File format read version
    pub file_format_read: u8,
    /// Bytes of unused space at end of each page
    pub reserved_bytes_per_page: u8,
    /// Maximum embedded payload fraction (must be 64)
    pub max_payload_fraction: u8,
    /// Minimum embedded payload fraction (must be 32)
    pub min_payload_fraction: u8,
    /// Leaf payload fraction (must be 32)
    pub leaf_payload_fraction: u8,
    /// File change counter
    pub file_change_counter: u32,
    /// Size of database in pages
    pub database_size_pages: u32,
    /// Page number of first freelist trunk page
    pub first_freelist_page: u32,
    /// Total number of freelist pages
    pub freelist_page_count: u32,
    /// Schema cookie
    pub schema_cookie: u32,
    /// Schema format number (1, 2, 3, or 4)
    pub schema_format: u32,
    /// Default page cache size
    pub default_cache_size: u32,
    /// Page number of largest root btree page (auto-vacuum)
    pub largest_root_page: u32,
    /// Text encoding
    pub text_encoding: TextEncoding,
    /// User version
    pub user_version: u32,
    /// Incremental vacuum mode flag
    pub incremental_vacuum: u32,
    /// Application ID
    pub application_id: u32,
    /// Version-valid-for number
    pub version_valid_for: u32,
    /// SQLite version number
    pub sqlite_version: u32,
}

impl DatabaseHeader {
    /// Get the usable page size (page_size - reserved_bytes_per_page)
    pub fn usable_size(&self) -> u32 {
        self.page_size - self.reserved_bytes_per_page as u32
    }

    /// Format SQLite version as string (e.g., "3.39.0")
    pub fn sqlite_version_string(&self) -> String {
        let major = self.sqlite_version / 1_000_000;
        let minor = (self.sqlite_version / 1_000) % 1_000;
        let patch = self.sqlite_version % 1_000;
        format!("{}.{}.{}", major, minor, patch)
    }
}
