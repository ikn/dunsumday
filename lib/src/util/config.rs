use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use crate::db::{ConfigId, Db, DbResult, StoredConfig, StoredItem, StoredOcc};
use crate::types::{Config, Item, ItemType, TaskCompletionConfig};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ResolvedConfig {
    pub id: ConfigId,
    pub scope_config: Config,
    pub resolved_config: Config,
    pub parent: Box<Option<ResolvedConfig>>,
}

pub fn build_config_ids_all() -> Vec<ConfigId> {
    vec![ConfigId::All]
}

pub fn build_config_ids_type(type_: ItemType) -> Vec<ConfigId> {
    let mut result = build_config_ids_all();
    result.push(ConfigId::Type(type_));
    result
}

pub fn build_config_ids_category(item: &Item) -> Vec<ConfigId> {
    let mut result = build_config_ids_type(item.type_);
    if let Some(cat) = &item.category {
        result.push(ConfigId::Category(cat.to_owned()));
    }
    result
}

pub fn build_config_ids_item(item: &StoredItem) -> Vec<ConfigId> {
    let mut result = build_config_ids_category(&item.item);
    result.push(ConfigId::Item { id: item.id.to_owned() });
    result
}

pub fn build_config_ids_occ(item: &StoredItem, occ: &StoredOcc)
-> Vec<ConfigId> {
    let mut result = build_config_ids_item(item);
    result.push(ConfigId::Occ { id: occ.id.to_owned() });
    result
}

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

/// configs are in the same order as returned by build_config_ids_*, i.e. parent
/// first
/// returns None if configs empty
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

/// result has no entry for items with no configs
pub fn get_items_configs<'i>(db: &impl Db, items: &[&'i StoredItem])
-> DbResult<Vec<(&'i StoredItem, ResolvedConfig)>> {
    let ids_by_item = items.iter()
        .map(|item| (*item, build_config_ids_item(item)))
        .collect::<Vec<_>>();
    get_objects_configs(db, &ids_by_item)
}

/// None if item has no configs
pub fn get_item_config(db: &impl Db, item: &StoredItem)
-> DbResult<Option<ResolvedConfig>> {
    let results = get_items_configs(db, &[item])?;
    Ok(results.into_iter().map(|(item, config)| config).next())
}

/// result has no entry for occs with no configs
pub fn get_occs_configs<'o>(
    db: &impl Db, occs: &[(&StoredItem, &'o StoredOcc)],
) -> DbResult<Vec<(&'o StoredOcc, ResolvedConfig)>> {
    let ids_by_occ = occs.iter()
        .map(|(item, occ)| (*occ, build_config_ids_occ(item, occ)))
        .collect::<Vec<_>>();
    get_objects_configs(db, &ids_by_occ)
}

/// None if occ has no configs
pub fn get_occ_config(db: &impl Db, item: &StoredItem, occ: &StoredOcc)
-> DbResult<Option<ResolvedConfig>> {
    let results = get_occs_configs(db, &[(item, occ)])?;
    Ok(results.into_iter().map(|(occ, config)| config).next())
}
