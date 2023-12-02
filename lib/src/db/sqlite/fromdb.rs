use std::str::FromStr;
use chrono::TimeZone;
use rusqlite::Row;
use crate::types::{Item, Config, ItemType, Occ, OccDate};
use crate::db::{ConfigId, DbResult, StoredItem, StoredConfig, StoredOcc};
use super::dbtypes;

pub const CONFIG_ID_ALL_DB_VALUE: u8 = 0;

pub fn internal_err<T>(r: rusqlite::Result<T>) -> DbResult<T> {
    r.map_err(|e| format!("internal error: {e}"))
}

pub fn internal_err_fn<T, F>(f: F) -> DbResult<T>
where
    F: FnOnce() -> rusqlite::Result<T>
{
    internal_err(f())
}

fn serde<T>(bytes: &[u8]) -> DbResult<T>
where
    T: serde::de::DeserializeOwned,
{
    rmp_serde::from_read(bytes)
        .map_err(|e| format!(
            "error deserialising value from database: {e}"))
}

pub fn row_get<T>(r: &Row, i: usize) -> DbResult<T>
where
    T: rusqlite::types::FromSql
{
    internal_err(r.get(i))
}

pub fn id(id: dbtypes::Id) -> String {
    id.to_string()
}

pub fn item_type(type_str: &str) -> DbResult<ItemType> {
    ItemType::from_str(type_str)
        .map_err(|e| format!(
            "error reading item type from database ({type_str}): {e}"))
}

pub const ITEMS_SQL: &str = "id, created_date, updated_date, type, active, \
                             category, name, desc, sched_blob";
pub const ITEMS_CREATED_COL: &str = "created_date";

/// for result selected by [`ITEMS_SQL`]
pub fn item(r: &Row) -> DbResult<StoredItem> {
    let type_str: String = row_get(r, 3)?;
    let sched_bytes: Vec<u8> = row_get(r, 8)?;
    Ok(StoredItem {
        id: id(row_get(r, 0)?),
        created: occ_date(r, 1)?,
        updated: occ_date(r, 2)?,
        item: Item {
            type_: item_type(&type_str)?,
            active: row_get(r, 4)?,
            category: row_get(r, 5)?,
            name: row_get(r, 6)?,
            desc: row_get(r, 7)?,
            sched: serde(&sched_bytes)?,
        },
    })
}

pub fn occ_date(r: &Row, i: usize) -> DbResult<OccDate> {
    let epoch_s = row_get(r, i)?;
    let naive = chrono::NaiveDateTime::from_timestamp_opt(epoch_s, 0)
        .ok_or("read invalid date value (column index {i}): {epoch_s}")?;
    Ok(chrono::Utc.from_utc_datetime(&naive))
}

pub const OCCS_SQL: &str = "id, item_id, active, start_date, end_date, \
                            task_completion_progress";
pub const OCCS_START_COL: &str = "start_date";

/// for result selected by [`OCCS_SQL`]
pub fn occ_data(r: &Row) -> DbResult<(String, StoredOcc)> {
    let item_id: String = id(row_get(r, 1)?);
    let occ = StoredOcc {
        id: id(row_get(r, 0)?),
        occ: Occ {
            active: row_get(r, 2)?,
            start: occ_date(r, 3)?,
            end: occ_date(r, 4)?,
            task_completion_progress: row_get(r, 5)?,
        },
    };
    Ok((item_id, occ))
}

/// for result selected by [`OCCS_SQL`]
pub fn occ(r: &Row) -> DbResult<StoredOcc> {
    Ok(occ_data(r)?.1)
}

pub const CONFIGS_SQL: &str = "id_all, id_type, id_category, id_item, id_occ, \
                               config_blob";

/// for result selected by [`CONFIGS_SQL`]
pub fn config(r: &Row) -> DbResult<StoredConfig> {
    let bytes: Vec<u8> = row_get(r, 5)?;
    let config: Config = serde(&bytes)?;

    let id_all: Option<u8> = row_get(r, 0)?;
    let id_type = row_get::<Option<String>>(r, 1)?
        .map(|t| item_type(t.as_ref())).transpose()?;
    let id_cat: Option<String> = row_get(r, 2)?;
    let id_item = row_get::<Option<dbtypes::Id>>(r, 3)?.map(id);
    let id_occ = row_get::<Option<dbtypes::Id>>(r, 4)?.map(id);

    let id = if id_all == Some(CONFIG_ID_ALL_DB_VALUE) {
        Ok(ConfigId::All)
    } else if let Some(type_) = id_type {
        Ok(ConfigId::Type(type_))
    } else if let Some(cat) = id_cat {
        Ok(ConfigId::Category(cat))
    } else if let Some(id) = id_item {
        Ok(ConfigId::Item { id })
    } else if let Some(id) = id_occ {
        Ok(ConfigId::Occ { id })
    } else {
        Err("".to_owned())
    }?;

    Ok(StoredConfig { id, config: config })
}
