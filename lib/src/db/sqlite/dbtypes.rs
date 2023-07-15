use crate::db::DbResult;

pub const SCHEMA_FILES: [&str; 1] = ["00-init.sql"];

pub type Id = i64;
pub type InsertResult = DbResult<String>;

pub mod table {
    pub const ITEMS: &str = "tbl_items";
    pub const OCCS: &str = "tbl_occs";
    pub const CONFIGS: &str = "tbl_configs";
}
