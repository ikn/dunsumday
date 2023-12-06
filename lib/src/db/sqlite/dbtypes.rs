//! General data and types for this module.

/// Names of SQL files read to initialise database schema.
pub const SCHEMA_FILES: [&str; 1] = ["00-init.sql"];

/// Unique ID of an object stored in the database, internal to
/// [`sqlite`](crate::db::sqlite).
pub type Id = i64;

/// Database table names.
pub mod table {
    pub const ITEMS: &str = "tbl_items";
    pub const OCCS: &str = "tbl_occs";
    pub const CONFIGS: &str = "tbl_configs";
}
