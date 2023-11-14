use std::collections::HashMap;
use std::fs;
use std::path::Path;
use rusqlite::Connection;
use crate::types::{Item, OccDate, Occ};
use crate::db::{ConfigId, DbResult, DbResults, DbWriteResult, DbUpdate, IdToken,
                SortDirection, Stored, StoredConfig, UpdateId};

mod dbtypes;
mod fromdb;
mod read;
mod todb;
mod write;

#[derive(Debug)]
pub struct Db { conn: Connection }

fn init_schema(conn: &Connection, schema_path: &Path) -> DbResult<()> {
    dbtypes::SCHEMA_FILES.iter()
        .try_for_each(|filename| {
            let path = schema_path.join(filename);
            let sql = fs::read_to_string(&path)
                .map_err(|e| format!("error reading schema file ({}): {e}",
                                     path.display()))?;
            conn.execute_batch(&sql)
                .map_err(|e| format!(
                    "error executing schema file ({}): {e}",
                    path.display()))
        })
}

pub fn open(db_path: &Path, schema_path: &Path)
-> DbResult<impl crate::db::Db> {
    let db_path_parent = db_path.parent()
        .map(|p| if p.as_os_str().is_empty() { Path::new(".") } else { p })
        .unwrap_or(db_path);

    fs::create_dir_all(db_path_parent)
        .map_err(|e| format!("error creating directory ({}): {e}",
                             db_path_parent.display()))?;
    let conn = Connection::open(db_path)
        .map_err(|e| format!("error opening database ({}): {e}",
                             db_path.display()))?;
    fromdb::internal_err(rusqlite::vtab::array::load_module(&conn))?;
    init_schema(&conn, schema_path)?;
    Ok(Db { conn })
}

fn resolve_update_id<'a>(
    ids_map: &'a HashMap<IdToken, String>,
    id: &'a UpdateId,
) -> DbResult<&'a str> {
    match id {
        UpdateId::Id(id) => Ok(id),
        UpdateId::Token(token) => {
            ids_map.get(token)
                .map(|id| id.as_ref())
                .ok_or(format!("invalid update token ({token}): \
                                not part of this write, \
                                or used before created"))
        }
    }
}

fn write_update(
    conn: &Connection,
    ids_map: &HashMap<IdToken, String>,
    update: &DbUpdate,
) -> DbResult<Option<(IdToken, String)>> {
    match update {
        DbUpdate::CreateItem { id_token, item } => {
            write::create_item(conn, item)
                .map(|id| Some((*id_token, id)))
        }
        DbUpdate::UpdateItem(item) => {
            write::update_item(conn, item).map(|_| None)
        }
        DbUpdate::DeleteItem { id } => {
            write::delete_item(conn, id).map(|_| None)
        }
        DbUpdate::SetConfig(config) => {
            write::set_config(conn, config).map(|_| None)
        }
        DbUpdate::DeleteConfig { id: config_id } => {
            write::delete_config(conn, config_id).map(|_| None)
        }
        DbUpdate::CreateOcc { id_token, item_id, occ } => {
            let item_id = resolve_update_id(ids_map, item_id)?;
            write::create_occ(conn, item_id, occ)
                .map(|id| Some((*id_token, id)))
        }
        DbUpdate::UpdateOcc(occ) => {
            write::update_occ(conn, occ).map(|_| None)
        }
        DbUpdate::DeleteOcc { id } => {
            write::delete_occ(conn, id).map(|_| None)
        }
    }
}

impl crate::db::Db for Db {
    fn write(&mut self, updates: &[&DbUpdate]) -> DbWriteResult {
        let mut ids_map: HashMap<IdToken, String> = HashMap::new();
        let tx = self.conn.transaction()
            .map_err(|e| format!("error writing to database: {e}"))?;

        for update in updates {
            write_update(&tx, &ids_map, update)?
                .and_then(|id_map| {
                    ids_map.insert(id_map.0, id_map.1)
                });
        }

        tx.commit()
            .map_err(|e| format!("error writing to database: {e}"))?;
        Ok(ids_map)
    }

    fn get_all_items(&self) -> DbResults<Stored<Item>> {
        read::get_all_items(&self.conn)
    }

    fn get_items(&self, ids: &[&str]) -> DbResults<Stored<Item>> {
        read::get_items(&self.conn, todb::multi(todb::id, ids)?)
    }

    fn get_configs(&self, ids: &[&ConfigId])
    -> DbResults<StoredConfig> {
        read::get_configs(&self.conn, ids)
    }

    fn get_occs(&self, ids: &[&str]) -> DbResults<Stored<Occ>> {
        read::get_occs(&self.conn, todb::multi(todb::id, ids)?)
    }

    fn find_occs(
        &self,
        item_ids: &[&str],
        start: Option<&OccDate>,
        end: Option<&OccDate>,
        sort: SortDirection,
        max_results: Option<u32>,
    ) -> DbResult<HashMap<String, Vec<Stored<Occ>>>> {
        let item_dbids = todb::multi(todb::id, item_ids)?;
        read::find_occs(&self.conn, item_dbids, start, end, sort, max_results)
    }
}
