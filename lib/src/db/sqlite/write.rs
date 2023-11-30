use rusqlite::{Connection, named_params};
use crate::db::{ConfigId, DbResult, Stored, StoredConfig};
use crate::types::{Item, Occ};
use super::dbtypes::{self, table::{CONFIGS, ITEMS, OCCS}};
use super::{fromdb, todb};

pub fn create_item(conn: &Connection, item: &Item)
-> dbtypes::InsertResult {
    conn.execute(format!("
        INSERT INTO {ITEMS} (type, active, category, name, desc, sched_blob)
        VALUES (:type, :active, :cat, :name, :desc, :sched_blob)
    ").as_ref(), named_params! {
        ":type": todb::item_type(&item.type_),
        ":active": item.active,
        ":cat": item.category,
        ":name": item.name,
        ":desc": item.desc,
        ":sched_blob": todb::sched(&item.sched)?,
    })
        .map(|_| fromdb::id(conn.last_insert_rowid()))
        .map_err(|e| format!("error creating item ({item:?}): {e}"))
}

pub fn update_item(conn: &Connection, item: &Stored<Item>)
-> DbResult<()> {
    conn.execute(format!("
        UPDATE {ITEMS}
        SET type = :type, active = :active, category = :cat, name = :name,
            desc = :desc
        WHERE id = :id
    ").as_ref(), named_params! {
        ":id": todb::id(&item.id)?,
        ":type": todb::item_type(&item.data.type_),
        ":active": item.data.active,
        ":cat": item.data.category,
        ":name": item.data.name,
        ":desc": item.data.desc,
    })
        .map(|_| ())
        .map_err(|e| format!("error updating item ({item:?}): {e}"))
}

pub fn delete_item(conn: &Connection, id: &str) -> DbResult<()> {
    conn.execute(format!("
        DELETE FROM {ITEMS}
        WHERE id = :id
    ").as_ref(), named_params! {
        ":id": todb::id(id)?,
    })
        .map(|_| ())
        .map_err(|e| format!("error deleting item ({id:?}): {e}"))
}

pub fn set_config(conn: &Connection, config: &StoredConfig)
-> dbtypes::InsertResult {
    let mut id_all: Option<u8> = None;
    let mut id_type: Option<&str> = None;
    let mut id_cat: Option<&str> = None;
    let mut id_item: Option<dbtypes::Id> = None;
    let mut id_occ: Option<dbtypes::Id> = None;

    match &config.id {
        ConfigId::All => { id_all = Some(fromdb::CONFIG_ID_ALL_DB_VALUE); }
        ConfigId::Type(type_) => { id_type = Some(todb::item_type(type_)); }
        ConfigId::Category(cat) => { id_cat = Some(cat); }
        ConfigId::Item { id } => { id_item = Some(todb::id(id)?); }
        ConfigId::Occ { id } => { id_occ = Some(todb::id(id)?); }
    }

    conn.execute(format!("
        INSERT INTO {CONFIGS}
            (id_all, id_type, id_category, id_item, id_occ, config_blob)
        VALUES
            (:id_all, :id_type, :id_category, :id_item, :id_occ, :config_blob)
    ").as_ref(), named_params! {
        ":id_all": id_all,
        ":id_type": id_type,
        ":id_category": id_cat,
        ":id_item": id_item,
        ":id_occ": id_occ,
        ":config_blob": todb::config(&config.data)?,
    })
        .map(|_| fromdb::id(conn.last_insert_rowid()))
        .map_err(|e| format!("error setting config ({config:?}): {e}"))
}

pub fn delete_config(conn: &Connection, id: &ConfigId) -> DbResult<()> {
    let mut id_all: Option<u8> = None;
    let mut id_type: Option<&str> = None;
    let mut id_cat: Option<&str> = None;
    let mut id_item: Option<dbtypes::Id> = None;
    let mut id_occ: Option<dbtypes::Id> = None;

    let id_param = match id {
        ConfigId::All => {
            id_all = Some(fromdb::CONFIG_ID_ALL_DB_VALUE);
            ":id_all"
        }
        ConfigId::Type(type_) => {
            id_type = Some(todb::item_type(type_));
            ":id_type"
        }
        ConfigId::Category(cat) => {
            id_cat = Some(cat);
            ":id_cat"
        }
        ConfigId::Item { id } => {
            id_item = Some(todb::id(id)?);
            ":id_item"
        }
        ConfigId::Occ { id } => {
            id_occ = Some(todb::id(id)?);
            ":id_occ"
        }
    };

    conn.execute(format!("
        DELETE FROM {CONFIGS}
        WHERE id = {id_param}
    ").as_ref(), named_params! {
        ":id_all": id_all,
        ":id_type": id_type,
        ":id_category": id_cat,
        ":id_item": id_item,
        ":id_occ": id_occ,
    })
        .map(|_| ())
        .map_err(|e| format!("error deleting item ({id:?}): {e}"))
}

pub fn create_occ(conn: &Connection, item_id: &str, occ: &Occ)
-> dbtypes::InsertResult {
    conn.execute(format!("
        INSERT INTO {OCCS}
            (item_id, active, start_date, end_date, task_completion_progress)
        VALUES
            (:item_id, :active, :start, :end, :progress)
    ").as_ref(), named_params! {
        ":item_id": todb::id(item_id)?,
        ":active": occ.active,
        ":start": todb::occ_date(&occ.start),
        ":end": todb::occ_date(&occ.end),
        ":progress": occ.task_completion_progress,
    })
        .map(|_| fromdb::id(conn.last_insert_rowid()))
        .map_err(|e| format!("error creating occurrence ({occ:?}): {e}"))
}

pub fn update_occ(conn: &Connection, occ: &Stored<Occ>)
-> DbResult<()> {
    conn.execute(format!("
        UPDATE {OCCS}
        SET active = :active, start_date = :start, end_date = :end,
            task_completion_progress = :progress
        WHERE id = :id
    ").as_ref(), named_params! {
        ":id": todb::id(&occ.id)?,
        ":active": occ.data.active,
        ":start": todb::occ_date(&occ.data.start),
        ":end": todb::occ_date(&occ.data.end),
        ":progress": occ.data.task_completion_progress,
    })
        .map(|_| ())
        .map_err(|e| format!("error updating occurrence ({occ:?}): {e}"))
}

pub fn delete_occ(conn: &Connection, id: &str) -> DbResult<()> {
    conn.execute(format!("
        DELETE FROM {OCCS}
        WHERE id = :id
    ").as_ref(), named_params! {
        ":id": todb::id(id)?,
    })
        .map(|_| ())
        .map_err(|e| format!("error deleting occurrence ({id:?}): {e}"))
}
