use crate::config::ValueRef;

pub const DB_SQLITE_PATH: ValueRef<'_> = ValueRef {
    names: &["db", "sqlite", "db-path"],
    def: "/var/lib/dunsumday/db.sqlite",
};

pub const DB_SQLITE_SCHEMA_PATH: ValueRef<'_> = ValueRef {
    names: &["db", "sqlite", "schema-path"],
    def: "/usr/share/dunsumday/db-schema",
};
