use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqliteVizError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid SQLite magic header")]
    InvalidMagic,

    #[error("Invalid WAL magic header: {0:#x}")]
    InvalidWalMagic(u32),

    #[error("Invalid page type: {0:#x}")]
    InvalidPageType(u8),

    #[error("Invalid page number: {0}")]
    InvalidPageNumber(u32),

    #[error("Invalid varint encoding")]
    InvalidVarint,

    #[error("Invalid serial type: {0}")]
    InvalidSerialType(u64),

    #[error("Invalid text encoding: {0}")]
    InvalidTextEncoding(u32),

    #[error("Page out of bounds: page {page} requested, but database has {total} pages")]
    PageOutOfBounds { page: u32, total: u32 },

    #[error("Unexpected end of data while parsing {context}")]
    UnexpectedEof { context: &'static str },

    #[error("Schema parse error: {0}")]
    SchemaError(String),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SqliteVizError>;
