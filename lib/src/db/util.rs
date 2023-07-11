use crate::types::{Item, Config as DbConfig, ConfigId, Occ, Sched};
use super::{Db, DbResult, DbResults, DbUpdate, UpdateId};

fn get_single_helper<T>(id: &str, r: DbResults<T>) -> DbResult<T> {
    r.map(|is| is.into_iter().next())
        .transpose()
        .unwrap_or(Err(format!("object with given ID does not exist: {id}")))
}

pub fn create_item(db: &mut impl Db, item: &Item) -> DbResult<String> {
    let id_token = DbUpdate::id_token();
    let mut ids = db.write(&[&DbUpdate::create_item(&id_token, item)])?;
    ids.remove(&id_token)
        .ok_or("unknown error - ID not returned".to_owned())
}

pub fn update_item(db: &mut impl Db, item: &Item) -> DbResult<()> {
    if let Some(id) = &item.id {
        db.write(&[&DbUpdate::update_item(id, item)])?;
        Ok(())
    } else {
        Err(format!("cannot update an item without an id ({item:?})"))
    }
}

pub fn delete_item(db: &mut impl Db, id: &str) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_item(id)])?;
    Ok(())
}

pub fn set_config(db: &mut impl Db, config: &DbConfig) -> DbResult<()> {
    if let Some(id) = &config.id {
        db.write(&[&DbUpdate::set_config(id.clone(), config)])?;
        Ok(())
    } else {
        Err(format!("cannot update a config without an id ({config:?})"))
    }
}

pub fn delete_config(db: &mut impl Db, id: &ConfigId) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_config(id.clone())])?;
    Ok(())
}

pub fn create_sched(db: &mut impl Db, item_id: &str, sched: &Sched)
-> DbResult<String> {
    let id_token = DbUpdate::id_token();
    let mut ids = db.write(&[
        &DbUpdate::create_sched(&id_token, UpdateId::Id(item_id), sched),
    ])?;
    ids.remove(&id_token)
        .ok_or("unknown error - ID not returned".to_owned())
}

pub fn update_sched(db: &mut impl Db, sched: &Sched) -> DbResult<()> {
    if let Some(id) = &sched.id {
        db.write(&[&DbUpdate::update_sched(id, sched)])?;
        Ok(())
    } else {
        Err(format!("cannot update a schedule without an id ({sched:?})"))
    }
}

pub fn delete_sched(db: &mut impl Db, id: &str) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_sched(id)])?;
    Ok(())
}

pub fn create_occ(db: &mut impl Db, sched_id: &str, occ: &Occ)
-> DbResult<String> {
    let id_token = DbUpdate::id_token();
    let mut ids = db.write(&[
        &DbUpdate::create_occ(&id_token, UpdateId::Id(sched_id), occ),
    ])?;
    ids.remove(&id_token)
        .ok_or("unknown error - ID not returned".to_owned())
}

pub fn update_occ(db: &mut impl Db, occ: &Occ) -> DbResult<()> {
    if let Some(id) = &occ.id {
        db.write(&[&DbUpdate::update_occ(id, occ)])?;
        Ok(())
    } else {
        Err(format!("cannot update an occurrence without an id ({occ:?})"))
    }
}

pub fn delete_occ(db: &mut impl Db, id: &str) -> DbResult<()> {
    db.write(&[&DbUpdate::delete_occ(id)])?;
    Ok(())
}

pub fn get_item(db: &impl Db, id: &str) -> DbResult<Item> {
    get_single_helper(id, db.get_items(&[id]))
}

pub fn get_config(db: &impl Db, id: &ConfigId) -> DbResult<Option<DbConfig>> {
    db.get_configs(&[id]).map(|mut cs| cs.remove(id))
}

pub fn get_sched(db: &impl Db, id: &str) -> DbResult<Sched> {
    get_single_helper(id, db.get_scheds(&[id]))
}

pub fn get_occ(db: &impl Db, id: &str) -> DbResult<Occ> {
    get_single_helper(id, db.get_occs(&[id]))
}
