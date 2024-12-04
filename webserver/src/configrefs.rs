use dunsumday::config::ValueRef;

pub const UI_PATH: ValueRef<'_> = ValueRef {
    names: &["webserver", "paths", "ui"],
    def: "/usr/share/dunsumday/webserver/resources/ui",
};

pub const SERVER_ALL_INTERFACES: ValueRef<'_> = ValueRef {
    names: &["webserver", "server", "all-interfaces"],
    def: "true",
};

pub const SERVER_PORT: ValueRef<'_> = ValueRef {
    names: &["webserver", "server", "port"],
    def: "26300",
};

pub const SERVER_ROOT_PATH: ValueRef<'_> = ValueRef {
    names: &["webserver", "server", "root-path"],
    def: "/",
};

pub const SERVER_API_PATH: ValueRef<'_> = ValueRef {
    names: &["webserver", "server", "paths", "api"],
    def: "/api",
};

pub const SERVER_UI_PATH: ValueRef<'_> = ValueRef {
    names: &["webserver", "server", "paths", "ui"],
    def: "/ui",
};
