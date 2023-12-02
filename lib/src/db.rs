use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::configrefs;
use crate::types::{Config as DbConfig, Item, ItemType, Occ, OccDate};

mod sqlite;
pub mod util;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StoredItem {
    pub id: String,
    pub created: OccDate,
    pub updated: OccDate,
    pub item: Item,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StoredOcc {
    pub id: String,
    pub occ: Occ,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ConfigId {
    // in inheritance order, parent first
    All,
    Type(ItemType),
    Category(String),
    Item { id: String },
    Occ { id: String },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StoredConfig {
    pub id: ConfigId,
    pub config: DbConfig,
}

pub type DbResult<T> = Result<T, String>;
pub type DbWriteResult = DbResult<HashMap<IdToken, String>>;
pub type DbResults<T> = DbResult<Vec<T>>;

pub type IdToken = u64;
static UPDATE_TOKEN: atomic::AtomicU64 = atomic::AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UpdateId<'a> {
    Id(&'a str),
    Token(IdToken),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DbUpdate<'a> {
    CreateItem { id_token: IdToken, item: &'a Item },
    UpdateItem(&'a StoredItem),
    DeleteItem { id: &'a str },
    SetConfig(&'a StoredConfig),
    DeleteConfig { id: ConfigId },
    CreateOcc { id_token: IdToken, item_id: UpdateId<'a>, occ: &'a Occ },
    UpdateOcc(&'a StoredOcc),
    DeleteOcc { id: &'a str },
}

impl<'a> DbUpdate<'a> {
    pub fn id_token() -> IdToken {
        UPDATE_TOKEN.fetch_add(1, atomic::Ordering::Relaxed)
    }

    pub fn create_item(id_token: IdToken, item: &'a Item) -> DbUpdate<'a> {
        DbUpdate::CreateItem { id_token, item }
    }

    pub fn update_item(item: &'a StoredItem) -> DbUpdate<'a> {
        DbUpdate::UpdateItem(item)
    }

    pub fn delete_item(id: &'a str) -> DbUpdate<'a> {
        DbUpdate::DeleteItem { id }
    }

    pub fn set_config(config: &'a StoredConfig) -> DbUpdate<'a> {
        DbUpdate::SetConfig(config)
    }

    pub fn delete_config(id: ConfigId) -> DbUpdate<'a> {
        DbUpdate::DeleteConfig { id }
    }

    pub fn create_occ(
        id_token: IdToken,
        item_id: UpdateId<'a>,
        occ: &'a Occ
    ) -> DbUpdate<'a> {
        DbUpdate::CreateOcc { id_token, item_id, occ }
    }

    pub fn update_occ(occ: &'a StoredOcc) -> DbUpdate<'a> {
        DbUpdate::UpdateOcc(occ)
    }

    pub fn delete_occ(id: &'a str) -> DbUpdate<'a> {
        DbUpdate::DeleteOcc { id }
    }
}

pub trait Db {
    fn write(&mut self, updates: &[&DbUpdate]) -> DbWriteResult;

    fn find_items(&self, active: Option<bool>, start: Option<&OccDate>)
    -> DbResults<StoredItem>;

    fn get_items(&self, ids: &[&str]) -> DbResults<StoredItem>;

    fn get_configs(&self, ids: &[&ConfigId]) -> DbResults<StoredConfig>;

    fn get_occs(&self, ids: &[&str]) -> DbResults<StoredOcc>;

    /// results are keyed by item ID
    /// results are ordered by date before applying max_results
    fn find_occs(
        &self,
        item_ids: &[&str],
        start: Option<&OccDate>,
        end: Option<&OccDate>,
        sort: SortDirection,
        max_results: Option<u32>,
    ) -> DbResult<HashMap<String, Vec<StoredOcc>>>;
}

pub fn open(cfg: &impl Config) -> Result<impl Db, String> {
    sqlite::open(
        Path::new(cfg.get_ref(&configrefs::DB_SQLITE_PATH)),
        Path::new(cfg.get_ref(&configrefs::DB_SQLITE_SCHEMA_PATH)))
}
