use crate::config::{ValueRef, parse};

pub const UI_PATH: ValueRef<'_, String> = ValueRef {
    names: &["webserver", "paths", "ui"],
    def: "/usr/share/dunsumday/webserver/resources/ui",
    type_: &parse::STRING,
    validators: vec![],
};

pub const SERVER_ALL_INTERFACES: ValueRef<'_, String> = ValueRef {
    names: &["webserver", "server", "all-interfaces"],
    def: "true",
    type_: &parse::STRING,
    validators: vec![],

};

pub const SERVER_PORT: ValueRef<'_, String> = ValueRef {
    names: &["webserver", "server", "port"],
    def: "26300",
    type_: &parse::STRING,
    validators: vec![],
};

pub const SERVER_ROOT_PATH: ValueRef<'_, String> = ValueRef {
    names: &["webserver", "server", "root-path"],
    def: "/",
    type_: &parse::STRING,
    validators: vec![],
};

pub const SERVER_API_PATH: ValueRef<'_, String> = ValueRef {
    names: &["webserver", "server", "paths", "api"],
    def: "/api",
    type_: &parse::STRING,
    validators: vec![],
};

pub const SERVER_UI_PATH: ValueRef<'_, String> = ValueRef {
    names: &["webserver", "server", "paths", "ui"],
    def: "/ui",
    type_: &parse::STRING,
    validators: vec![],
};
