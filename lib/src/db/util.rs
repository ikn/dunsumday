//! Utilities for interacting with the database.

use crate::types::{Item, Occ};
use super::{ConfigId, Db, DbResult, DbResults, DbUpdate, StoredConfig,
            StoredItem, StoredOcc, UpdateId};

/// Extract the only result from the results of a lookup by ID.
fn get_single_helper<T>(id: &str, r: DbResults<T>) -> DbResult<T> {
    r.map(|results| results.into_iter().next())
        .transpose()
        .unwrap_or(Err(format!("object with given ID does not exist: {id}")))
}

/// Create an item.
pub fn create_item(db: &mut impl Db, item: Item) -> DbResult<StoredItem> {
    let id_token = DbUpdate::id_token();
    let mut ids = db.write(&[&DbUpdate::create_item(id_token, &item)])?;
    let id = ids.remove(&id_token)
        .ok_or("unknown error - ID not returned".to_owned())?;
    get_item(db, &id)
}

/// Update an item to be the same as the provided `item`.
pub fn update_item(db: &mut impl Db, item: &StoredItem) -> DbResult<()> {
    db.write(&[&DbUpdate::update_item(item)])?;
    Ok(())
}

/// Delete an item, succeeding if it doesn't exist.
pub fn delete_item(db: &mut impl Db, id: &str) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_item(id)])?;
    Ok(())
}

/// Create or update a config.
pub fn set_config(db: &mut impl Db, config: &StoredConfig) -> DbResult<()> {
    db.write(&[&DbUpdate::set_config(config)])?;
    Ok(())
}

/// Delete a config, succeeding if it doesn't exist.
pub fn delete_config(db: &mut impl Db, id: &ConfigId) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_config(id.clone())])?;
    Ok(())
}

/// Create an occurrence for the item with the given ID.
pub fn create_occ(db: &mut impl Db, item_id: &str, occ: &Occ)
-> DbResult<String> {
    let id_token = DbUpdate::id_token();
    let mut ids = db.write(&[
        &DbUpdate::create_occ(id_token, UpdateId::Id(item_id), occ),
    ])?;
    ids.remove(&id_token)
        .ok_or("unknown error - ID not returned".to_owned())
}

/// Update an occurrence to be the same as the provided `occ`.
pub fn update_occ(db: &mut impl Db, occ: &StoredOcc) -> DbResult<()> {
    db.write(&[&DbUpdate::update_occ(occ)])?;
    Ok(())
}

/// Delete an occurrence, succeeding if it doesn't exist.
pub fn delete_occ(db: &mut impl Db, id: &str) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_occ(id)])?;
    Ok(())
}

/// Get an existing item by ID.
pub fn get_item(db: &impl Db, id: &str) -> DbResult<StoredItem> {
    get_single_helper(id, db.get_items(&[id]))
}

/// Get an existing config by ID.
pub fn get_config(db: &impl Db, id: &ConfigId)
-> DbResult<Option<StoredConfig>> {
    db.get_configs(&[id]).map(|cs| cs.into_iter().next())
}

/// Get an existing occurrence by ID.
pub fn get_occ(db: &impl Db, id: &str) -> DbResult<StoredOcc> {
    get_single_helper(id, db.get_occs(&[id]))
}
