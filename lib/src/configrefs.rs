//! Configuration value references for configuration used by this library.

use crate::config::ValueRef;

/// SQLite database file path.
pub const DB_SQLITE_PATH: ValueRef<'_> = ValueRef {
    names: &["db", "sqlite", "db-path"],
    def: "/var/lib/dunsumday/db.sqlite",
};

/// SQLite database schema file path.
pub const DB_SQLITE_SCHEMA_PATH: ValueRef<'_> = ValueRef {
    names: &["db", "sqlite", "schema-path"],
    def: "/usr/local/share/dunsumday/lib/db-schema",
};
