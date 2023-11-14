use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use crate::db::{Db, DbResult, SortDirection, Stored};
use crate::types::Occ;
use super::config::{self, ResolvedConfig};

pub struct TaskProgress {
    progress: u32,
    total: u32,
    provided_excess: u32,
    received_excess: u32,
}

impl Default for TaskProgress {
    fn default() -> TaskProgress {
        TaskProgress {
            progress: 0,
            total: 1,
            provided_excess: 0,
            received_excess: 0,
        }
    }
}

/// returns remaining excess
fn transfer_progress(
    excess: u32,
    recv_prog_detail: &mut TaskProgress,
) -> u32 {
    let needed = recv_prog_detail.total +
        recv_prog_detail.received_excess -
        recv_prog_detail.progress;
    let transfer = max(0, min(needed, excess));
    recv_prog_detail.received_excess += transfer;
    excess - transfer
}

/// occs must not contain duplicates
/// occs must all be for the same item
/// excess transfer prioritises the nearest occ
pub fn resolve_occs_progress_using(occs: &[(&Occ, &ResolvedConfig)])
-> HashMap<Occ, TaskProgress> {
    let mut results: HashMap<Occ, TaskProgress> = HashMap::new();
    let mut occs_excess: HashMap<Occ, u32> = HashMap::new();
    // (recipient, donor, distance)
    let mut donations = Vec::<(&Occ, &Occ, chrono::Duration)>::new();

    for (i, (recv_occ, config)) in occs.iter().enumerate() {
        let prog_detail = TaskProgress {
            progress: recv_occ.task_completion_progress,
            total: config.resolved_config
                .task_completion_conf.total.unwrap_or(1),
            ..Default::default()
        };
        occs_excess.insert((*recv_occ).clone(),
            recv_occ.task_completion_progress - prog_detail.total);
        results.insert((*recv_occ).clone(), prog_detail);

        let cmpl_cfg = &config.resolved_config.task_completion_conf;
        let excess_past_min = recv_occ.start - cmpl_cfg.excess_past_chrono();
        let excess_future_max = recv_occ.end + cmpl_cfg.excess_future_chrono();
        for (donor_occ, _) in occs {
            if donor_occ == recv_occ {
                continue
            }
            if donor_occ.start < recv_occ.start &&
               donor_occ.end > excess_past_min
            {
                donations.push((&recv_occ, &donor_occ,
                                recv_occ.start - donor_occ.end));
            } else if donor_occ.start > recv_occ.start &&
               donor_occ.start < excess_past_min
            {
                donations.push((&recv_occ, &donor_occ,
                                donor_occ.start - recv_occ.end));
            }
        }
    }

    donations.sort_unstable_by(|
        (a_recv_occ, a_donor_occ, a_dist),
        (b_recv_occ, b_donor_occ, b_dist),
    | {
        (a_dist, a_recv_occ.start, a_donor_occ.start)
            .cmp(&(b_dist, b_recv_occ.start, b_donor_occ.start))
    });

    for (recv_occ, donor_occ, _) in donations {
        let excess = occs_excess.get_mut(donor_occ).unwrap();
        let recv_prog_detail = results.get_mut(recv_occ).unwrap();
        *excess = transfer_progress(*excess, recv_prog_detail);
    }

    results
}

/// occs is item_id -> occs
fn expand_occs_for_progress(
    db: &impl Db,
    occs: &mut HashMap<String, HashSet<Occ>>,
    configs: &mut HashMap<Occ, ResolvedConfig>,
) -> DbResult<()> {
    let item_ids: Vec<&str> = occs.keys()
        .map(|i| i.as_str()).collect();

    let start = occs.iter()
        .flat_map(|(i, i_occs)| i_occs.iter())
        .map(|o| {
            configs.get(o).map(|c| {
                o.start - c.resolved_config
                    .task_completion_conf.excess_past_chrono()
            })
        })
        .min()
        .flatten();
    let end = occs.iter()
        .flat_map(|(i, i_occs)| i_occs.iter())
        .map(|o| {
            configs.get(o).map(|c| {
                o.end + c.resolved_config
                    .task_completion_conf.excess_future_chrono()
            })
        })
        .max()
        .flatten();

    if let (Some(start), Some(end)) = (start, end) {
        // update occs
        let retrieved_occs = db.find_occs(
            &item_ids, Some(&start), Some(&end), SortDirection::Asc, None)?;
        let mut new_occs: Vec<(&str, &Stored<Occ>)> = vec![];
        for (item_id, retrieved_item_occs) in &retrieved_occs {
            let item_occs = occs.entry(item_id.clone()).or_default();
            for retrieved_occ in retrieved_item_occs {
                if item_occs.insert(retrieved_occ.data.clone()) {
                    new_occs.push((&item_id, &retrieved_occ));
                }
            }
        }

        // update configs
        let new_item_ids = new_occs.iter().map(|(i, o)| *i).collect::<Vec<_>>();
        let new_items = db.get_items(&new_item_ids[..])?
            .into_iter()
            .map(|i| (i.id.clone(), i))
            .collect::<HashMap<_, _>>();
        let new_items_occs = new_occs.into_iter()
            .flat_map(|(id, o)| new_items.get(id).map(|i| (i, o)))
            .collect::<Vec<_>>();
        for (occ, config) in
        config::get_occs_configs(db, &new_items_occs[..])? {
            configs.insert(occ.data.clone(), config);
        }
    }
    Ok(())
}

/// occs keys are item IDs
/// may include other occs in result
pub fn resolve_occs_progress(
    db: &impl Db,
    occs: &[(&str, Vec<(&Occ, &ResolvedConfig)>)],
) -> DbResult<HashMap<Occ, TaskProgress>> {
    let mut expanded_occs: HashMap<String, HashSet<Occ>> = HashMap::new();
    let mut configs: HashMap<Occ, ResolvedConfig> = HashMap::new();
    for (item_id, occs_configs) in occs {
        let mut item_occs: HashSet<Occ> = HashSet::new();
        for (occ, config) in occs_configs {
            item_occs.insert((*occ).clone());
            configs.insert((*occ).clone(), (*config).clone());
        }
        expanded_occs.insert((*item_id).to_owned(), item_occs);
    }

    // We need all the occs that may affect the requested occs via excess
    // donation.  The call below gets all occs within excess donation range of
    // our occs.  Excess donation prioritises nearer donor occs, so if we expand
    // twice, we have enough information to know if a possible donor will find
    // a preferable recipient in the other direction.
    expand_occs_for_progress(db, &mut expanded_occs, &mut configs)?;
    expand_occs_for_progress(db, &mut expanded_occs, &mut configs)?;

    let occs_configs = configs.iter().collect::<Vec<_>>();
    // TODO: split into items first (probably make a restrict_map function and use below too?)
    let mut occs_progress = resolve_occs_progress_using(&occs_configs[..]);

    // only return the requested occs - progress may be incorrect for others
    let mut result = HashMap::<Occ, TaskProgress>::new();
    for (item_id, occs_configs) in occs {
        for (occ, config) in occs_configs {
            if let Some(progress) = occs_progress.remove(occ) {
                result.insert((*occ).clone(), progress);
            }
        }
    }
    Ok(result)
}

pub fn resolve_occ_progress(
    db: &impl Db,
    item_id: &str,
    occ: &Occ,
    config: &ResolvedConfig,
) -> DbResult<TaskProgress> {
    let results = resolve_occs_progress(db, &[
        (item_id, Vec::from([(occ, config)])),
    ])?;
    Ok(results.into_iter()
        .map(|(occ, progress)| progress)
        .next()
        .unwrap_or(Default::default()))
}
