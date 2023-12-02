use crate::types::{Item, Occ};
use super::{ConfigId, Db, DbResult, DbResults, DbUpdate, StoredConfig,
            StoredItem, StoredOcc, UpdateId};

fn get_single_helper<T>(id: &str, r: DbResults<T>) -> DbResult<T> {
    r.map(|results| results.into_iter().next())
        .transpose()
        .unwrap_or(Err(format!("object with given ID does not exist: {id}")))
}

pub fn create_item(db: &mut impl Db, item: Item) -> DbResult<StoredItem> {
    let id_token = DbUpdate::id_token();
    let mut ids = db.write(&[&DbUpdate::create_item(id_token, &item)])?;
    let id = ids.remove(&id_token)
        .ok_or("unknown error - ID not returned".to_owned())?;
    get_item(db, &id)
}

pub fn update_item(db: &mut impl Db, item: &StoredItem) -> DbResult<()> {
    db.write(&[&DbUpdate::update_item(item)])?;
    Ok(())
}

pub fn delete_item(db: &mut impl Db, id: &str) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_item(id)])?;
    Ok(())
}

pub fn set_config(db: &mut impl Db, config: &StoredConfig) -> DbResult<()> {
    db.write(&[&DbUpdate::set_config(config)])?;
    Ok(())
}

pub fn delete_config(db: &mut impl Db, id: &ConfigId) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_config(id.clone())])?;
    Ok(())
}

pub fn create_occ(db: &mut impl Db, item_id: &str, occ: &Occ)
-> DbResult<String> {
    let id_token = DbUpdate::id_token();
    let mut ids = db.write(&[
        &DbUpdate::create_occ(id_token, UpdateId::Id(item_id), occ),
    ])?;
    ids.remove(&id_token)
        .ok_or("unknown error - ID not returned".to_owned())
}

pub fn update_occ(db: &mut impl Db, occ: &StoredOcc) -> DbResult<()> {
    db.write(&[&DbUpdate::update_occ(occ)])?;
    Ok(())
}

pub fn delete_occ(db: &mut impl Db, id: &str) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_occ(id)])?;
    Ok(())
}

pub fn get_item(db: &impl Db, id: &str) -> DbResult<StoredItem> {
    get_single_helper(id, db.get_items(&[id]))
}

pub fn get_config(db: &impl Db, id: &ConfigId)
-> DbResult<Option<StoredConfig>> {
    db.get_configs(&[id]).map(|cs| cs.into_iter().next())
}

pub fn get_occ(db: &impl Db, id: &str) -> DbResult<StoredOcc> {
    get_single_helper(id, db.get_occs(&[id]))
}
