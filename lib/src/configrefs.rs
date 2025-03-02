//! Configuration value references for configuration used by this library.

use std::path::PathBuf;
use crate::config::{ValueRef, parse};

/// SQLite database file path.
pub const DB_SQLITE_PATH: ValueRef<'_, PathBuf> = ValueRef {
    names: &["db", "sqlite", "db-path"],
    def: "/var/lib/dunsumday/db.sqlite",
    type_: &parse::FILE_PATH,
    validators: &[],
};

/// SQLite database schema file path.
pub const DB_SQLITE_SCHEMA_PATH: ValueRef<'_, PathBuf> = ValueRef {
    names: &["db", "sqlite", "schema-path"],
    def: "/usr/local/share/dunsumday/lib/db-schema",
    type_: &parse::FILE_PATH,
    validators: &[],
};
