//! General high-level utilities.

use std::collections::HashMap;
use chrono::offset::Utc;
use crate::db::{Db, DbResult, DbResults, DbUpdate, IdToken, UpdateId,
                SortDirection, StoredItem, StoredOcc};
use crate::types::{Occ, OccDate, Sched};
use self::config::ResolvedConfig;

mod occgen;
pub mod config;
pub mod progress;
pub mod sched;

/// Determine whether `occ` is valid as an item's "current occurrence", relative
/// to the given `date`.
fn occ_is_current(date: OccDate, sched: &Sched, occ: &Occ) -> bool {
    match sched {
        Sched::Event(_) => occ.start >= date,
        _ => occ.start <= date && occ.end >= date,
    }
}

/// Get the "current occurrence" for each of the given `items`, relative to the
/// given `date`.
///
/// Not every item has a current occurrence.  For events, this is the next
/// occurrence.
pub fn get_items_current_occ<'i>(
    db: &mut impl Db,
    date: OccDate,
    items: &[&'i StoredItem]
) -> DbResult<Vec<(&'i StoredItem, StoredOcc)>> {
    let mut new_occs = HashMap::<IdToken, (&str, Occ)>::new();
    let mut items_last_token = Vec::<(&StoredItem, IdToken)>::new();
    let mut items_last_occ = Vec::<(&StoredItem, StoredOcc)>::new();

    for item in items {
        let occ_gen: Box<dyn occgen::OccGen> = match &item.item.sched {
            Sched::Event(sched) => Box::new(occgen::EventOccGen { sched }),
            Sched::ProgressTask(sched) =>
                Box::new(occgen::ProgressTaskOccGen { sched }),
            Sched::DeadlineTask(sched) =>
                Box::new(occgen::DeadlineTaskOccGen { sched }),
        };

        let mut item_occs = db.find_occs(
            &[&item.id], None, None, SortDirection::Desc, 1)?;
        let item_occ = item_occs.remove(&item.id)
            .and_then(|mut occs| occs.pop());
        let mut item_new_occs = match &item_occ {
            Some(occ) => occ_gen.generate_after(&occ.occ, date),
            None => occ_gen.generate_first(date).iter().cloned().collect(),
        };

        if !item_new_occs.is_empty() {
            // sort so last will become current
            item_new_occs.sort_by_key(|occ| occ.start);
            let mut last_token = 0;
            for occ in item_new_occs {
                last_token = DbUpdate::id_token();
                new_occs.insert(last_token, (&item.id, occ));
            }
            items_last_token.push((item, last_token));
        } else {
            // no new occs: current is the one we already found
            if let Some(item_occ_value) = item_occ {
                items_last_occ.push((item, item_occ_value));
            }
        }
    }

    let mut updates = Vec::new();
    for (id_token, (item_id, occ)) in &new_occs {
        updates.push(DbUpdate::create_occ(
            *id_token, UpdateId::Id(item_id), occ));
    }
    let update_refs: Vec<&DbUpdate> = updates.iter().collect();
    let mut new_occ_ids = db.write(&update_refs[..])?;
    for (item, id_token) in items_last_token {
        if let Some(occ_id) = new_occ_ids.remove(&id_token) {
            if let Some((_, occ)) = new_occs.remove(&id_token) {
                items_last_occ.push((item, StoredOcc { id: occ_id, occ }));
            }
        }
    }

    Ok(items_last_occ.iter()
        .filter(|(i, o)| occ_is_current(date, &i.item.sched, &o.occ))
        .cloned()
        .collect())
}

/// Get the "current occurrence" for an `item`, relative to the given `date`.
///
/// See [`get_items_current_occ`] for details.
pub fn get_item_current_occ(
    db: &mut impl Db,
    date: OccDate,
    item: &StoredItem,
) -> DbResult<Option<StoredOcc>> {
    let results = get_items_current_occ(db, date, &[item])?;
    Ok(results.into_iter()
        .map(|(item, occ)| occ)
        .next())
}

/// Get all "current" items along with their "current occurrence".
///
/// This returns all active items, excluding those with no occurrences after the
/// given `date`.
pub fn get_current_items(db: &mut impl Db, date: OccDate)
-> DbResults<(StoredItem, StoredOcc)> {
    let items = db.find_items(
        Some(true), Some(date), SortDirection::Asc, u32::MAX)?;
    let item_refs: Vec<&StoredItem> = items.iter().collect();
    let mut occs_by_item = get_items_current_occ(db, date, &item_refs)?
        .into_iter().collect::<HashMap<_, _>>();
    // can't move items and occs into the same value until we drop the returned
    // item refs
    let mut occs_by_item_index: HashMap<usize, StoredOcc> = items.iter()
        .enumerate()
        .flat_map(|(index, item)| {
            occs_by_item.remove(item).map(|occ| (index, occ)).into_iter()
        }).collect();
    Ok(items.into_iter()
        .enumerate()
        .flat_map(|(index, item)| {
            occs_by_item_index.remove(&index).map(|occ| (item, occ)).into_iter()
        }).collect())
}

/// Determine whether `date` is in `occ`'s alert period, according to the
/// `config`.
pub fn in_alert_period(occ: &Occ, config: &ResolvedConfig, date: OccDate)
-> bool {
    let alert_start = occ.end - config.resolved_config.occ_alert_chrono();
    let now = Utc::now();
    now >= alert_start && now < occ.end
}
