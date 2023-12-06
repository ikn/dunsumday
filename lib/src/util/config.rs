//! [Config]-related utilities.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use crate::db::{ConfigId, Db, DbResult, StoredConfig, StoredItem, StoredOcc};
use crate::types::{Config, Item, ItemType, TaskCompletionConfig};

/// A config associated with the scope it applies to, with all values resolved
/// by inheriting from parent scopes where applicable.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ResolvedConfig {
    pub id: ConfigId,
    /// The normal config that applies to this scope.
    pub scope_config: Config,
    /// `scope_config` with missing values filled in from parent scopes.
    pub resolved_config: Config,
    /// The most direct parent config that exists.
    pub parent: Box<Option<ResolvedConfig>>,
}

/// Get config IDs relevant to [`ConfigId::All`].
pub fn build_config_ids_all() -> Vec<ConfigId> {
    vec![ConfigId::All]
}

/// Get config IDs relevant to [`ConfigId::Type`].
pub fn build_config_ids_type(type_: ItemType) -> Vec<ConfigId> {
    let mut result = build_config_ids_all();
    result.push(ConfigId::Type(type_));
    result
}

/// Get config IDs relevant to [`ConfigId::Category`].
pub fn build_config_ids_category(item: &Item) -> Vec<ConfigId> {
    let mut result = build_config_ids_type(item.type_);
    if let Some(cat) = &item.category {
        result.push(ConfigId::Category(cat.to_owned()));
    }
    result
}

/// Get config IDs relevant to [`ConfigId::Item`].
pub fn build_config_ids_item(item: &StoredItem) -> Vec<ConfigId> {
    let mut result = build_config_ids_category(&item.item);
    result.push(ConfigId::Item { id: item.id.to_owned() });
    result
}

/// Get config IDs relevant to [`ConfigId::Occ`].
pub fn build_config_ids_occ(item: &StoredItem, occ: &StoredOcc)
-> Vec<ConfigId> {
    let mut result = build_config_ids_item(item);
    result.push(ConfigId::Occ { id: occ.id.to_owned() });
    result
}

/// Fill in missing values in the `child` config where they are present in the
/// `parent` config.
pub fn resolve_config_direct(parent: &Config, child: &Config) -> Config {
    let pcompl = &parent.task_completion_conf;
    let ccompl = &child.task_completion_conf;
    Config {
        occ_alert: child.occ_alert.or(parent.occ_alert),
        task_completion_conf: TaskCompletionConfig {
            total: ccompl.total.or(pcompl.total),
            unit: ccompl.unit.clone().or(pcompl.unit.clone()),
            excess_past: ccompl.excess_past.or(pcompl.excess_past),
            excess_future: ccompl.excess_future.or(pcompl.excess_future),
        },
    }
}

/// Resolve configs by filling in defaults from parents, given all the parents.
///
/// `configs` is all the configuration applying to a specific scope and its
/// parents, in order from parent to child.  This is the same order as returned
/// by the `build_config_ids_...` methods in this module.
///
/// Returns `None` when `configs` is empty.
pub fn resolve_config(configs: &[StoredConfig]) -> Option<ResolvedConfig> {
    if configs.is_empty() {
        None
    } else {
        let config = configs.first().unwrap();
        let mut resolved = ResolvedConfig {
            id: config.id.clone(),
            scope_config: config.config.clone(),
            resolved_config: config.config.clone(),
            parent: Box::new(None),
        };

        for config in configs {
            resolved = ResolvedConfig {
                id: config.id.clone(),
                resolved_config: resolve_config_direct(
                    &resolved.resolved_config, &config.config),
                scope_config: config.config.clone(),
                parent: Box::new(Some(resolved)),
            };
        }

        Some(resolved)
    }
}

/// Retrieve and resolve all configs for multiple objects.
///
/// `ids_by_obj` specifies the config IDs to try to retrieve for each object of
/// type `T`.  Objects with no stored config are not included in the result.
fn get_objects_configs<'t, T>(
    db: &impl Db,
    ids_by_obj: &[(&'t T, Vec<ConfigId>)],
) -> DbResult<Vec<(&'t T, ResolvedConfig)>>
where
    T: Clone + Eq + Hash
{
    let all_ids = ids_by_obj.iter()
        .flat_map(|(obj, ids)| ids)
        .collect::<HashSet<_>>()
        .into_iter().collect::<Vec<_>>();
    let mut config_by_id: HashMap<ConfigId, StoredConfig> =
        db.get_configs(&all_ids)?
            .into_iter()
            .map(|c| (c.id.to_owned(), c))
            .collect();

    let config_by_obj = ids_by_obj.iter()
        .flat_map(|(obj, ids)| {
            let configs = ids.iter()
                .flat_map(|id| config_by_id.remove(id))
                .collect::<Vec<_>>();
            resolve_config(&configs[..]).map(|rc| (*obj, rc))
        })
        .collect();
    Ok(config_by_obj)
}

/// Retrieve and resolve all configs for multiple items.
///
/// Items with no stored config are not included in the result.
pub fn get_items_configs<'i>(db: &impl Db, items: &[&'i StoredItem])
-> DbResult<Vec<(&'i StoredItem, ResolvedConfig)>> {
    let ids_by_item = items.iter()
        .map(|item| (*item, build_config_ids_item(item)))
        .collect::<Vec<_>>();
    get_objects_configs(db, &ids_by_item)
}

/// Retrieve and resolve configs for an item.
///
/// The result is `None` when the item has no stored config.
pub fn get_item_config(db: &impl Db, item: &StoredItem)
-> DbResult<Option<ResolvedConfig>> {
    let results = get_items_configs(db, &[item])?;
    Ok(results.into_iter().map(|(item, config)| config).next())
}

/// Retrieve and resolve all configs for multiple occurrences.
///
/// Occurrences with no stored config are not included in the result.
pub fn get_occs_configs<'o>(
    db: &impl Db, occs: &[(&StoredItem, &'o StoredOcc)],
) -> DbResult<Vec<(&'o StoredOcc, ResolvedConfig)>> {
    let ids_by_occ = occs.iter()
        .map(|(item, occ)| (*occ, build_config_ids_occ(item, occ)))
        .collect::<Vec<_>>();
    get_objects_configs(db, &ids_by_occ)
}

/// Retrieve and resolve configs for an occurrence.
///
/// The result is `None` when the occurrence has no stored config.
pub fn get_occ_config(db: &impl Db, item: &StoredItem, occ: &StoredOcc)
-> DbResult<Option<ResolvedConfig>> {
    let results = get_occs_configs(db, &[(item, occ)])?;
    Ok(results.into_iter().map(|(occ, config)| config).next())
}
