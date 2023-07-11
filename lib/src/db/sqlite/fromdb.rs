use std::str::FromStr;
use rusqlite::Row;
use crate::types::{Item, Config, ConfigId, ItemType, Occ, OccDate,
                   Sched};
use crate::db::DbResult;
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

pub const ITEMS_SQL: &str = "id, type, category, desc";

/// for result selected by [`ITEMS_SHALLOW_SQL`]
pub fn item(r: &Row) -> DbResult<Item> {
    let type_str: String = row_get(r, 1)?;
    Ok(Item {
        id: Some(id(row_get(r, 0)?)),
        type_: item_type(&type_str)?,
        category: row_get(r, 2)?,
        desc: row_get(r, 3)?,
    })
}

pub const SCHEDS_SQL: &str = "id, item_id, sched_blob";

/// for result selected by [`SCHEDS_SQL`]
pub fn sched_data(r: &Row) -> DbResult<(String, Sched)> {
    let id = id(row_get(r, 0)?);
    let item_id: String = self::id(row_get(r, 1)?);

    let bytes: Vec<u8> = row_get(r, 2)?;
    let mut sched: Sched = serde(&bytes[..])?;
    sched.id = Some(id);

    Ok((item_id, sched))
}

/// for result selected by [`SCHEDS_SQL`]
pub fn sched(r: &Row) -> DbResult<Sched> {
    Ok(sched_data(r)?.1)
}

pub fn occ_date(r: &Row, i: usize) -> DbResult<OccDate> {
    let epoch_s = row_get(r, i)?;
    let naive = chrono::NaiveDateTime::from_timestamp_opt(epoch_s, 0)
        .ok_or("read invalid date value (column index {i}): {epoch_s}")?;
    Ok(chrono::DateTime::from_utc(naive, chrono::offset::Utc))
}

pub const OCCS_SQL: &str = "id, sched_id, start_date, end_date, \
                            task_completion_progress";

/// for result selected by [`OCCS_SQL`]
pub fn occ_data(r: &Row) -> DbResult<(String, Occ)> {
    let sched_id: String = id(row_get(r, 1)?);
    let occ = Occ {
        id: Some(id(row_get(r, 0)?)),
        start: occ_date(r, 3)?,
        end: occ_date(r, 4)?,
        task_completion_progress: row_get(r, 5)?,
    };
    Ok((sched_id, occ))
}

/// for result selected by [`OCCS_SQL`]
pub fn occ(r: &Row) -> DbResult<Occ> {
    Ok(occ_data(r)?.1)
}

pub const CONFIGS_SQL: &str = "id_all, id_type, id_category, id_item, id_occ, \
                               config_blob";

/// for result selected by [`CONFIGS_SQL`]
pub fn config(r: &Row) -> DbResult<Config> {
    let bytes: Vec<u8> = row_get(r, 5)?;
    let mut config: Config = serde(&bytes[..])?;

    let id_all: Option<u8> = row_get(r, 0)?;
    let id_type = row_get::<Option<String>>(r, 1)?
        .map(|t| item_type(t.as_ref())).transpose()?;
    let id_cat: Option<String> = row_get(r, 2)?;
    let id_item = row_get::<Option<dbtypes::Id>>(r, 3)?.map(id);
    let id_occ = row_get::<Option<dbtypes::Id>>(r, 4)?.map(id);

    if id_all == Some(CONFIG_ID_ALL_DB_VALUE) {
        config.id = Some(ConfigId::All);
    } else if let Some(type_) = id_type {
        config.id = Some(ConfigId::Type(type_));
    } else if let Some(cat) = id_cat {
        config.id = Some(ConfigId::Category(cat));
    } else if let Some(id) = id_item {
        config.id = Some(ConfigId::Item { id });
    } else if let Some(id) = id_occ {
        config.id = Some(ConfigId::Occ { id });
    }

    Ok(config)
}
