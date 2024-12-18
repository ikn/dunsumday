//! Helpers for reading from the database.

use std::collections::HashMap;
use std::rc::Rc;
use rusqlite::{Connection, named_params, ToSql, types::Value};
use crate::db::{ConfigId, DbResult, DbResults, SortDirection, StoredConfig,
                StoredItem, StoredOcc};
use crate::types::{ItemType, OccDate};
use super::dbtypes::table::{CONFIGS, ITEMS, OCCS};
use super::fromdb::{self, CONFIG_ID_ALL_DB_VALUE, CONFIGS_SQL,
                    ITEMS_CREATED_COL, ITEMS_SQL, OCCS_SQL, OCCS_START_COL};
use super::todb;

/// See [Db::find_items](crate::db::Db::find_items).
pub fn find_items(
    conn: &Connection,
    active: Option<bool>,
    start: Option<OccDate>,
    sort: SortDirection,
    max_results: u32,
) -> DbResults<StoredItem> {
    let mut exprs: Vec<String> = Vec::new();
    let mut params: Vec<(&str, &dyn ToSql)> = Vec::new();
    let active_value = active.unwrap_or(false);
    if active.is_some() {
        exprs.push("active = :active".to_owned());
        params.push((":active", &active));
    }
    let start_db_value = start.map(todb::occ_date).unwrap_or(0);
    if let Some(start) = start {
        exprs.push("only_occ_end > :min_end".to_owned());
        params.push((":min_end", &start_db_value));
    }
    let sort_sql = match sort {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };
    params.push((":max_results", &max_results));

    fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(format!("
            SELECT {ITEMS_SQL} from {ITEMS} WHERE {}
            ORDER BY {ITEMS_CREATED_COL} {sort_sql}
            LIMIT :max_results
        ", &exprs.join(", ")).as_ref())?;
        let rows = stmt.query_map(&params[..], todb::mapper(fromdb::item))?;
        rows.collect()
    })
}

/// See [Db::get_items](crate::db::Db::get_items).
pub fn get_items(conn: &Connection, dbids: Rc<Vec<Value>>)
-> DbResults<StoredItem> {
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

/// See [Db::get_configs](crate::db::Db::get_configs).
pub fn get_configs(conn: &Connection, ids: &[&ConfigId])
-> DbResults<StoredConfig> {
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
    fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(&stmts.join(" UNION "))?;
        let rows = stmt.query_map(params, todb::mapper(fromdb::config))?;
        rows.collect()
    })
}

/// See [Db::find_occs](crate::db::Db::find_occs).
pub fn find_occs(
    conn: &Connection,
    item_dbids: Rc<Vec<Value>>,
    start: Option<OccDate>,
    end: Option<OccDate>,
    sort: SortDirection,
    max_results: u32,
) -> DbResult<HashMap<String, Vec<StoredOcc>>> {
    let mut exprs: Vec<String> = Vec::new();
    let mut params: Vec<(&str, &dyn ToSql)> = Vec::new();
    if !item_dbids.is_empty() {
        exprs.push("item_id IN rarray(:item_ids)".to_owned());
        params.push((":item_ids", &item_dbids));
    }
    let start_db_value = start.map(todb::occ_date).unwrap_or(0);
    if let Some(start) = start {
        exprs.push("end_date > :min_end".to_owned());
        params.push((":min_end", &start_db_value));
    }
    let end_db_value = end.map(todb::occ_date).unwrap_or(0);
    if let Some(end) = end {
        exprs.push("start_date < :max_start".to_owned());
        params.push((":max_start", &end_db_value));
    }
    let sort_sql = match sort {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };
    params.push((":max_results", &max_results));


    let occs: Vec<(String, StoredOcc)> = fromdb::internal_err_fn(|| {
        let mut stmt = conn.prepare(format!("
            SELECT {OCCS_SQL} from {OCCS}
            WHERE ({})
            ORDER BY {OCCS_START_COL} {sort_sql}
            LIMIT :max_results
        ", &exprs.join(", ")).as_ref())?;
        let rows = stmt.query_map(&params[..], todb::mapper(fromdb::occ_data))?;
        rows.collect()
    })?;

    let mut result = HashMap::<String, Vec<StoredOcc>>::new();
    for (item_id, occ) in occs {
        result.entry(item_id).or_default().push(occ);
    }
    Ok(result)
}

/// See [Db::get_occs](crate::db::Db::get_occs).
pub fn get_occs(conn: &Connection, dbids: Rc<Vec<Value>>)
-> DbResults<StoredOcc> {
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
