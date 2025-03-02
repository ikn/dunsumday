//! Database for storing items, occurrences and configs.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic;
use serde::{Deserialize, Serialize};
use crate::config::{self, Config};
use crate::configrefs;
use crate::types::{Config as ItemConfig, Item, ItemType, Occ, OccDate};

mod sqlite;
pub mod util;

/// [`Item`] that has been stored in the database.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StoredItem {
    pub id: String,
    pub created: OccDate,
    pub updated: OccDate,
    pub item: Item,
}

/// [`Occ`] that has been stored in the database.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StoredOcc {
    pub id: String,
    pub occ: Occ,
}

/// The target of a [`Config`], also serving as a unique identifier.
///
/// Options are in order of precedence when applying to an occurrence---later
/// options take precedence over earlier options.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ConfigId {
    /// Applies to all occurrences.
    All,
    /// Applies to all occurrences of all items of this type.
    Type(ItemType),
    /// Applies to all occurrences of all items with this category.
    Category(String),
    /// Applies to all occurrences of the item with this `id`.
    Item { id: String },
    /// Applies to the occurrence with this `id`.
    Occ { id: String },
}

/// [`Config`] that has been stored in the database.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StoredConfig {
    pub id: ConfigId,
    pub config: ItemConfig,
}

/// The core `Result` type used by database functions.  All database errors
/// will be strings.
pub type DbResult<T> = Result<T, String>;
/// Created items and occurrences, as a map from token to database ID.
pub type DbWriteResult = DbResult<HashMap<IdToken, String>>;
pub type DbResults<T> = DbResult<Vec<T>>;

/// Temporary ID referring to objects that are yet to be written to the
/// database.
pub type IdToken = u64;
/// Used to generate `IdToken` values sequentially in a thread-safe manner.
static UPDATE_TOKEN: atomic::AtomicU64 = atomic::AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Reference to an object that may or may not have been written to the database
/// already.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UpdateId<'a> {
    Id(&'a str),
    Token(IdToken),
}

/// Describes an operation that modifies the database.
///
/// Functions are helpers for creating enum values.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DbUpdate<'a> {
    CreateItem { id_token: IdToken, item: &'a Item },
    UpdateItem(&'a StoredItem),
    DeleteItem { id: &'a str },
    /// [`Config`] identifiers are known before writing, so this is a
    /// create-or-update operation.
    SetConfig(&'a StoredConfig),
    DeleteConfig { id: ConfigId },
    CreateOcc { id_token: IdToken, item_id: UpdateId<'a>, occ: &'a Occ },
    UpdateOcc(&'a StoredOcc),
    DeleteOcc { id: &'a str },
}

impl<'a> DbUpdate<'a> {
    /// Generate a token for use as a temporary ID.
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

    /// [`Config`] identifiers are known before writing, so this is a
    /// create-or-update operation.
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

/// Database for storing items, occurrences and configs.
pub trait Db {
    /// Write some changes to the database.
    ///
    /// `updates` are processed in the order provided.  Tokens used must refer
    /// to objects created by a previous updated.
    ///
    /// Delete operations do not fail if the object doesn't exist.
    fn write(&mut self, updates: &[&DbUpdate]) -> DbWriteResult;

    /// Get all items matching the specified criteria.
    ///
    /// `active` filters to items which are active or not.  `start` filters to
    /// items which are recurring, or which are non-recurring and occur after
    /// this date.
    ///
    /// Results are ordered by created date, before applying `max_results`.
    fn find_items(
        &self,
        active: Option<bool>,
        start: Option<OccDate>,
        sort: SortDirection,
        max_results: u32,
    ) -> DbResults<StoredItem>;

    /// Get items with the given IDs.
    ///
    /// If an ID doesn't exist, the call succeeds and the item is missing from
    /// the results.
    fn get_items(&self, ids: &[&str]) -> DbResults<StoredItem>;

    /// Get configs with the given IDs.
    ///
    /// If an ID doesn't exist, the call succeeds and the config is missing from
    /// the results.
    fn get_configs(&self, ids: &[&ConfigId]) -> DbResults<StoredConfig>;

    /// Get occurrences with the given IDs.
    ///
    /// If an ID doesn't exist, the call succeeds and the occurrence is missing
    /// from the results.
    fn get_occs(&self, ids: &[&str]) -> DbResults<StoredOcc>;

    /// Get all occurrences matching the specified criteria.
    ///
    /// `start` and `end` filter to occurrences which overlap the time range.
    ///
    /// The results are a map from item ID to occurrences.  This may not contain
    /// an entry for requested items without any found occurrences.  Results are
    /// ordered by occurrence start date, before applying `max_results`.
    fn find_occs(
        &self,
        item_ids: &[&str],
        start: Option<OccDate>,
        end: Option<OccDate>,
        sort: SortDirection,
        max_results: u32,
    ) -> DbResult<HashMap<String, Vec<StoredOcc>>>;
}

/// Open a connection to the database.
pub fn open<C>(cfg: &C) -> Result<impl Db, String>
where
    C: Config + ?Sized,
{
    sqlite::open(
        Path::new(&config::get_ref(cfg, &configrefs::DB_SQLITE_PATH)?),
        Path::new(&config::get_ref(cfg, &configrefs::DB_SQLITE_SCHEMA_PATH)?))
}
