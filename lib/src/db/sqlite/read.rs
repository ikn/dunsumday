use std::collections::HashMap;
use std::rc::Rc;
use rusqlite::{Connection, named_params, types::Value};
use crate::db::{DbResult, DbResults};
use crate::types::{Item, ConfigId, Config, ItemType, OccDate, Occ};
use super::dbtypes::table::{CONFIGS, ITEMS, OCCS};
use super::fromdb::{self, CONFIG_ID_ALL_DB_VALUE, CONFIGS_SQL, ITEMS_SQL,
                    OCCS_SQL};
use super::todb;

pub fn get_all_items(conn: &Connection) -> DbResults<Item> {
    fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(format!("
            SELECT {ITEMS_SQL} from {ITEMS}
        ").as_ref())?;
        let rows = stmt.query_map((), todb::mapper(fromdb::item))?;
        rows.collect()
    })
}

pub fn get_items(conn: &Connection, dbids: Rc<Vec<Value>>) -> DbResults<Item> {
    fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(format!("
            SELECT {ITEMS_SQL} from {ITEMS}
            WHERE id IN rarray(:ids)
        ").as_ref())?;
        let rows = stmt.query_map(
            named_params! { ":ids": dbids },
            todb::mapper(fromdb::item))?;
        rows.collect()
    })
}

pub fn get_configs(conn: &Connection, ids: &[&ConfigId])
-> DbResult<HashMap<ConfigId, Config>> {
    let mut all: bool = false;
    let mut types: Vec<&ItemType> = Vec::new();
    let mut cats: Vec<&str> = Vec::new();
    let mut item_ids: Vec<&str> = Vec::new();
    let mut occ_ids: Vec<&str> = Vec::new();

    for id in ids {
        match id {
            ConfigId::All => { all = true; }
            ConfigId::Type(type_) => { types.push(type_); }
            ConfigId::Category(cat) => { cats.push(cat); }
            ConfigId::Item { id } => { item_ids.push(id); }
            ConfigId::Occ { id } => { occ_ids.push(id); }
        }
    }

    let mut stmts: Vec<String> = Vec::new();
    if all {
        stmts.push(format!("
            SELECT {CONFIGS_SQL} from {CONFIGS}
            WHERE id_all = {CONFIG_ID_ALL_DB_VALUE}
        ").to_owned());
    }
    if !types.is_empty() {
        stmts.push(format!("
            SELECT {CONFIGS_SQL} from {CONFIGS}
            WHERE id_type IN rarray(:types)
        ").to_owned());
    }
    if !types.is_empty() {
        stmts.push(format!("
            SELECT {CONFIGS_SQL} from {CONFIGS}
            WHERE id_category IN rarray(:cats)
        ").to_owned());
    }
    if !types.is_empty() {
        stmts.push(format!("
            SELECT {CONFIGS_SQL} from {CONFIGS}
            WHERE id_item IN rarray(:item_ids)
        ").to_owned());
    }
    if !types.is_empty() {
        stmts.push(format!("
            SELECT {CONFIGS_SQL} from {CONFIGS}
            WHERE id_occ IN rarray(:occ_ids)
        ").to_owned());
    }

    let params = named_params! {
        ":types": todb::multi(
            |type_| Ok(todb::item_type(type_).to_owned()),
            &types)?,
        ":cats": todb::multi(|c| Ok(c.to_owned()), &cats)?,
        ":item_ids": todb::multi(todb::id, &item_ids)?,
        ":occ_ids": todb::multi(todb::id, &occ_ids)?,
    };
    let configs: Vec<Config> = fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(&stmts.join(" UNION "))?;
        let rows = stmt.query_map(params, todb::mapper(fromdb::config))?;
        rows.collect()
    })?;

    let mut configs_map: HashMap<ConfigId, Config> = HashMap::new();
    for config in configs {
        if let Some(id) = config.id.clone() {
            configs_map.insert(id, config);
        }
    }
    Ok(configs_map)
}

pub fn find_occs(
    conn: &Connection,
    start: Option<&OccDate>,
    end: Option<&OccDate>,
    item_dbids: Rc<Vec<Value>>,
) -> DbResults<Occ> {
    let mut exprs: Vec<String> = Vec::new();

    if let Some(start) = start {
        exprs.push("end_date > :min_end".to_owned());
    }
    if let Some(end) = end {
        exprs.push("start_date < :max_start".to_owned());
    }

    if !item_dbids.is_empty() {
        exprs.push("item_id IN rarray(:item_ids)".to_owned());
    }

    let params = named_params! {
        ":min_end": start.map(|d| d.timestamp()).unwrap_or(0),
        ":max_start": end.map(|d| d.timestamp()).unwrap_or(0),
        ":item_ids": item_dbids,
    };

    fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(format!("
            SELECT {OCCS_SQL} from {OCCS}
            WHERE ({})
        ", &exprs.join(", ")).as_ref())?;
        let rows = stmt.query_map(params, todb::mapper(fromdb::occ))?;
        rows.collect()
    })
}

pub fn get_occs(conn: &Connection, dbids: Rc<Vec<Value>>) -> DbResults<Occ> {
    fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(format!("
            SELECT {OCCS_SQL} from {OCCS}
            WHERE id IN rarray(:ids)
        ").as_ref())?;
        let rows = stmt.query_map(
            named_params! { ":ids": dbids },
            todb::mapper(fromdb::occ))?;
        rows.collect()
    })
}
