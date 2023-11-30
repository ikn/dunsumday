use std::rc::Rc;
use chrono::NaiveTime;
use rusqlite::{Row, ToSql, types::Value};
use super::dbtypes;
use crate::db::{DbResult, DbResults};
use crate::types::{Config, DayFilter, ItemType, OccDate, Sched};

type Params<'a> = &'a [(&'a str, &'a dyn ToSql)];

fn serde<T>(val: &T) -> DbResult<Vec<u8>>
where
    T: serde::Serialize + std::fmt::Debug + ?Sized
{
    rmp_serde::to_vec(val)
        .map_err(|e| format!(
            "error serialising value for database ({val:?}): {e}"))
}

pub fn id(id: &str) -> DbResult<dbtypes::Id> {
    id.parse().map_err(|_| format!("invalid ID: {id}"))
}

pub fn multi<F, T, S>(f: F, values: &[&T]) -> DbResult<Rc<Vec<Value>>>
where
    F: Fn(&T) -> DbResult<S>,
    T: ?Sized,
    Value: From<S>,
{
    let dbvalues: DbResults<S> = values.iter().copied().map(f).collect();
    Ok(Rc::new(dbvalues?.into_iter().map(Value::from).collect()))
}

pub fn item_type(type_: &ItemType) -> &str {
    type_.as_ref()
}

pub fn item_only_occ_date(sched: &Sched) -> Option<i64> {
    match &sched {
        Sched::Event(event_sched) => {
            match event_sched.days {
                DayFilter::Date { dom, month, year } => {
                    Some(occ_date(
                        &event_sched.initial_day
                            .and_time(NaiveTime::MIN).and_utc()))
                },
                _ => None,
            }
        },
        _ => None,
    }
}

pub fn sched(sched: &Sched) -> DbResult<Vec<u8>> {
    serde(sched)
}

pub fn occ_date(date: &OccDate) -> i64 {
    date.timestamp()
}

pub fn config(config: &Config) -> DbResult<Vec<u8>> {
    serde(&config)
}

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
