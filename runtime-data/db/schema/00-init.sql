CREATE TABLE IF NOT EXISTS tbl_items (
    id INTEGER PRIMARY KEY,
    /* epoch seconds */
    created_date INTEGER NOT NULL,
    /* epoch seconds */
    updated_date INTEGER NOT NULL,
    type TEXT NOT NULL,
    active INTEGER NOT NULL,
    category TEXT,
    name TEXT NOT NULL,
    desc TEXT,
    /* MessagePack types::Sched */
    sched_blob BLOB NOT NULL,
    /* for non-recurring events, the end date of the only occurrence, in epoch seconds */
    only_occ_end INTEGER
);
CREATE INDEX IF NOT EXISTS idx_items_created_date
    ON tbl_items (created_date);

CREATE TABLE IF NOT EXISTS tbl_occs (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL,
    active INTEGER NOT NULL,
    /* epoch seconds */
    start_date INTEGER NOT NULL,
    /* epoch seconds */
    end_date INTEGER NOT NULL,
    task_completion_progress INTEGER NOT NULL,
    CONSTRAINT fk_occs_items
        FOREIGN KEY (item_id)
        REFERENCES tbl_items (id)
);
CREATE INDEX IF NOT EXISTS idx_occs_start_date
    ON tbl_occs (start_date);
CREATE INDEX IF NOT EXISTS idx_occs_end_date
    ON tbl_occs (end_date);

CREATE TABLE IF NOT EXISTS tbl_configs (
    /* 0 to enable for all items, else null */
    id_all INTEGER,
    id_type TEXT,
    id_category TEXT,
    id_item INTEGER,
    id_occ INTEGER,
    /* MessagePack types::Config */
    config_blob BLOB NOT NULL,
    CONSTRAINT idx_configs_id
        UNIQUE (id_all, id_type, id_category, id_item, id_occ)
        ON CONFLICT REPLACE,
    CONSTRAINT fk_configs_items
        FOREIGN KEY (id_item)
        REFERENCES tbl_items (id),
    CONSTRAINT fk_configs_occs
        FOREIGN KEY (id_occ)
        REFERENCES tbl_occs (id)
);
