use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic;
use crate::config::Config;
use crate::configrefs;
use crate::types::{Item, Config as DbConfig, ConfigId, Occ, OccDate, Sched};

mod sqlite;
pub mod util;

pub type DbResult<T> = Result<T, String>;
pub type DbWriteResult = DbResult<HashMap<IdToken, String>>;
pub type DbResults<T> = DbResult<Vec<T>>;

pub type IdToken = u64;
static UPDATE_TOKEN: atomic::AtomicU64 = atomic::AtomicU64::new(0);

#[derive(Debug)]
pub enum UpdateId<'a> {
    Id(&'a str),
    Token(IdToken),
}

#[derive(Debug)]
pub enum DbUpdate<'a> {
    CreateItem { id_token: &'a IdToken, item: &'a Item },
    UpdateItem { id: &'a str, item: &'a Item },
    DeleteItem { id: &'a str },
    SetConfig { id: ConfigId, config: &'a DbConfig },
    DeleteConfig { id: ConfigId },
    CreateSched { id_token: &'a IdToken, item_id: UpdateId<'a>,
                  sched: &'a Sched },
    UpdateSched { id: &'a str, sched: &'a Sched },
    DeleteSched { id: &'a str },
    CreateOcc { id_token: &'a IdToken, sched_id: UpdateId<'a>, occ: &'a Occ },
    UpdateOcc { id: &'a str, occ: &'a Occ },
    DeleteOcc { id: &'a str },
}

impl<'a> DbUpdate<'a> {
    pub fn id_token() -> IdToken {
        UPDATE_TOKEN.fetch_add(1, atomic::Ordering::Relaxed)
    }

    pub fn create_item(id_token: &'a IdToken, item: &'a Item) -> DbUpdate<'a> {
        DbUpdate::CreateItem { id_token, item }
    }

    pub fn update_item(id: &'a str, item: &'a Item) -> DbUpdate<'a> {
        DbUpdate::UpdateItem { id, item }
    }

    pub fn delete_item(id: &'a str) -> DbUpdate<'a> {
        DbUpdate::DeleteItem { id }
    }

    pub fn set_config(id: ConfigId, config: &'a DbConfig) -> DbUpdate<'a> {
        DbUpdate::SetConfig { id, config }
    }

    pub fn delete_config(id: ConfigId) -> DbUpdate<'a> {
        DbUpdate::DeleteConfig { id }
    }

    pub fn create_sched(
        id_token: &'a IdToken,
        item_id: UpdateId<'a>,
        sched: &'a Sched,
    ) -> DbUpdate<'a> {
        DbUpdate::CreateSched { id_token, item_id, sched }
    }

    pub fn update_sched(id: &'a str, sched: &'a Sched) -> DbUpdate<'a> {
        DbUpdate::UpdateSched { id, sched }
    }

    pub fn delete_sched(id: &'a str) -> DbUpdate<'a> {
        DbUpdate::DeleteSched { id }
    }

    pub fn create_occ(
        id_token: &'a IdToken,
        sched_id: UpdateId<'a>,
        occ: &'a Occ
    ) -> DbUpdate<'a> {
        DbUpdate::CreateOcc { id_token, sched_id, occ }
    }

    pub fn update_occ(id: &'a str, occ: &'a Occ) -> DbUpdate<'a> {
        DbUpdate::UpdateOcc { id, occ }
    }

    pub fn delete_occ(id: &'a str) -> DbUpdate<'a> {
        DbUpdate::DeleteOcc { id }
    }
}

pub trait Db {
    fn write(&mut self, updates: &[&DbUpdate]) -> DbWriteResult;

    fn get_all_items(&self) -> DbResults<Item>;

    fn get_items(&self, ids: &[&str]) -> DbResults<Item>;

    fn get_configs(&self, ids: &[&ConfigId])
    -> DbResult<HashMap<ConfigId, DbConfig>>;

    fn get_item_scheds(&self, item_id: &str) -> DbResults<Sched>;

    fn get_scheds(&self, ids: &[&str]) -> DbResults<Sched>;

    fn find_occs(
        &self,
        start: Option<&OccDate>,
        end: Option<&OccDate>,
        sched_ids: &[&str],
    ) -> DbResults<Occ>;

    fn get_occs(&self, ids: &[&str]) -> DbResults<Occ>;
}

pub fn open(cfg: &impl Config) -> Result<impl Db, String> {
    sqlite::open(
        Path::new(cfg.get_ref(&configrefs::DB_SQLITE_PATH)),
        Path::new(cfg.get_ref(&configrefs::DB_SQLITE_SCHEMA_PATH)))
}
