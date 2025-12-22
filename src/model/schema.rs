use serde::Serialize;

/// Type of object in the schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ObjectType {
    Table,
    Index,
    View,
    Trigger,
}

impl ObjectType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(ObjectType::Table),
            "index" => Some(ObjectType::Index),
            "view" => Some(ObjectType::View),
            "trigger" => Some(ObjectType::Trigger),
            _ => None,
        }
    }
}

/// An entry from sqlite_schema (sqlite_master)
#[derive(Debug, Clone, Serialize)]
pub struct SchemaEntry {
    /// Object type (table, index, view, trigger)
    pub object_type: ObjectType,
    /// Name of the object
    pub name: String,
    /// Name of the table this object is associated with
    pub table_name: String,
    /// Root page number for tables and indexes (0 for views/triggers)
    pub root_page: u32,
    /// SQL text that created this object
    pub sql: Option<String>,
}

/// Complete database schema
#[derive(Debug, Clone, Default, Serialize)]
pub struct Schema {
    /// All schema entries
    pub entries: Vec<SchemaEntry>,
}

impl Schema {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Get all tables in the schema
    pub fn tables(&self) -> impl Iterator<Item = &SchemaEntry> {
        self.entries.iter().filter(|e| e.object_type == ObjectType::Table)
    }

    /// Get all indexes in the schema
    pub fn indexes(&self) -> impl Iterator<Item = &SchemaEntry> {
        self.entries.iter().filter(|e| e.object_type == ObjectType::Index)
    }

    /// Get a table by name
    pub fn get_table(&self, name: &str) -> Option<&SchemaEntry> {
        self.tables().find(|e| e.name == name)
    }

    /// Get an index by name
    pub fn get_index(&self, name: &str) -> Option<&SchemaEntry> {
        self.indexes().find(|e| e.name == name)
    }

    /// Get all indexes for a given table
    pub fn indexes_for_table<'a>(&'a self, table_name: &'a str) -> impl Iterator<Item = &'a SchemaEntry> {
        self.indexes().filter(move |e| e.table_name == table_name)
    }
}
