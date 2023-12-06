//! Convert things from the external format to the format used in the database.

use std::rc::Rc;
use chrono::NaiveTime;
use rusqlite::{Row, types::Value};
use super::dbtypes;
use crate::db::{DbResult, DbResults};
use crate::types::{Config, DayFilter, ItemType, OccDate, Sched};

/// Serialise a serialisable value to bytes using MessagePack.
fn serde<T>(val: &T) -> DbResult<Vec<u8>>
where
    T: serde::Serialize + std::fmt::Debug + ?Sized
{
    rmp_serde::to_vec(val)
        .map_err(|e| format!(
            "error serialising value for database ({val:?}): {e}"))
}

/// Convert an external object ID to a database ID.
pub fn id(id: &str) -> DbResult<dbtypes::Id> {
    id.parse().map_err(|_| format!("invalid ID: {id}"))
}

/// Produce a SQLite prepared statement parameter for multiple `values`, first
/// mapping over failable function `f`.
pub fn multi<F, T, S>(f: F, values: &[&T]) -> DbResult<Rc<Vec<Value>>>
where
    F: Fn(&T) -> DbResult<S>,
    T: ?Sized,
    Value: From<S>,
{
    let dbvalues: DbResults<S> = values.iter().copied().map(f).collect();
    Ok(Rc::new(dbvalues?.into_iter().map(Value::from).collect()))
}

/// Convert item type to value stored in database.
pub fn item_type(type_: &ItemType) -> &str {
    type_.as_ref()
}

/// Produce a value for the `only_occ_date` column for an item.
pub fn item_only_occ_date(sched: &Sched) -> Option<i64> {
    match &sched {
        Sched::Event(event_sched) => {
            match event_sched.days {
                DayFilter::Date { dom, month, year } => {
                    Some(occ_date(
                        event_sched.initial_day
                            .and_time(NaiveTime::MIN).and_utc()))
                },
                _ => None,
            }
        },
        _ => None,
    }
}

/// Convert schedule to value stored in database.
pub fn sched(sched: &Sched) -> DbResult<Vec<u8>> {
    serde(sched)
}

/// Convert occurrence date to value stored in database.
pub fn occ_date(date: OccDate) -> i64 {
    date.timestamp()
}

/// Convert config to value stored in database.
pub fn config(config: &Config) -> DbResult<Vec<u8>> {
    serde(&config)
}

/// Convert a row-mapping function that produces [`DbResult`] to a row-mapping
/// function suitable for use with [`rusqlite::Statement::query_map`].
pub fn mapper<T, F>(f: F) -> impl Fn(&Row<'_>) -> rusqlite::Result<T>
where
    F: Fn(&Row<'_>) -> DbResult<T>,
{
    move |r| {
        f(r)
            .map_err(|e| rusqlite::Error::from(
                rusqlite::types::FromSqlError::InvalidType))
    }

}
