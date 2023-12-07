//! Utilities related to [task progress](Occ::task_completion_progress).

use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use crate::db::{Db, DbResult, SortDirection, StoredOcc};
use crate::types::Occ;
use super::config::{self, ResolvedConfig};

/// Progress details for a task, including donation information (see
/// [`excess_past`](crate::types::TaskCompletionConfig::excess_past),
/// [`excess_future`](crate::types::TaskCompletionConfig::excess_future)).
pub struct TaskProgress {
    /// Progress towards completing the occurrence.
    ///
    /// This may be greater than `total`.  This is the progress registered
    /// directly with this occurrence, before transferring progress between
    /// occurrences.
    progress: u32,
    /// Target occurrence completion amount.
    total: u32,
    /// Amount of `progress` donated to other occurrences.
    ///
    /// This occurs where transfer is allowed, and `progress` is greater than
    /// `total`.
    donated_excess: u32,
    /// Amount of `progress` received from other occurrences.
    ///
    /// This occurs where transfer is allowed, and `progress` is less than
    /// `total`.
    received_excess: u32,
}

impl Default for TaskProgress {
    fn default() -> TaskProgress {
        TaskProgress {
            progress: 0,
            total: 1,
            donated_excess: 0,
            received_excess: 0,
        }
    }
}

/// Transfer progress to `recv_prog_detail`, given `excess` progress available
/// to transfer.
///
/// Returns the new value for `excess` (remaining progress available to
/// transfer).
fn transfer_progress(
    excess: u32,
    recv_prog_detail: &mut TaskProgress,
) -> u32 {
    let needed = recv_prog_detail.total +
        recv_prog_detail.received_excess -
        recv_prog_detail.progress;
    let transfer = max(0, min(needed, excess));
    // TODO: donated_excess
    recv_prog_detail.received_excess += transfer;
    excess - transfer
}

/// Resolve progress for occurrences.
///
/// `occs` must all be for the same item, and must not contain duplicate
/// occurrences.  Only the given occurrences will be used as sources and targets
/// of progress transfer.
///
/// When transferring progress between occurrences, nearer donors are
/// prioritised.
fn resolve_occs_progress_using(occs: &[(&Occ, &ResolvedConfig)])
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

/// Modify `occs` and `configs` to add all occurrences within the total progress
/// transfer range of the initial `occs`.
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
            &item_ids, Some(start), Some(end),
            SortDirection::Asc, std::u32::MAX)?;
        let mut new_occs: Vec<(&str, &StoredOcc)> = vec![];
        for (item_id, retrieved_item_occs) in &retrieved_occs {
            let item_occs = occs.entry(item_id.clone()).or_default();
            for retrieved_occ in retrieved_item_occs {
                if item_occs.insert(retrieved_occ.occ.clone()) {
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
            configs.insert(occ.occ.clone(), config);
        }
    }
    Ok(())
}

/// Get progress details for the given occurrences.
///
/// `occs` is a slice of `(item_id, occs_and_configs)` pairs.
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

    let mut occs_progress = HashMap::<Occ, TaskProgress>::new();
    for (item_id, _) in occs {
        let item_occs_configs = expanded_occs.get(item_id.to_owned()).iter()
            .flat_map(|item_occs| item_occs.iter())
            .flat_map(|occ| configs.get(occ).map(|config| (occ, config)))
            .collect::<Vec<_>>();
        occs_progress.extend(
            resolve_occs_progress_using(&item_occs_configs[..]));
    }

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

/// Get progress details for `occ`.
///
/// `item_id` is the ID of the occurrence's item.  `config` is the occurrence's
/// config.
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
