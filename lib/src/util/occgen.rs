use chrono::{NaiveDate, NaiveTime};
use crate::types::{ProgressTaskSched, DeadlineTaskSched, EventSched, Occ,
                   OccDate};
use super::sched;

pub trait OccGen {
    /// generate occs after the given occ, up to the given date
    fn generate_after(&self, occ: &Occ, until: OccDate)
        -> Vec<Occ>;

    /// generate an occ as the first occ for the item, if any time is allowed,
    /// which will be the new current occ
    fn generate_first(&self, now: OccDate) -> Option<Occ>;
}

fn day_to_occ_date(day: NaiveDate) -> OccDate {
    day.and_time(NaiveTime::MIN).and_utc()
}

fn new_occ(start: OccDate, end: OccDate) -> Occ {
    Occ {
        start,
        end,
        task_completion_progress: 0,
    }
}

pub struct EventOccGen<'a> {
    pub sched: &'a EventSched,
}

impl EventOccGen<'_> {
    fn for_day(&self, day: NaiveDate) -> Occ {
        let start_time = self.sched.time.unwrap_or(NaiveTime::MIN);
        let start = day.and_time(start_time).and_utc();
        new_occ(start, start)
    }
}

impl OccGen for EventOccGen<'_> {
    fn generate_after(&self, occ: &Occ, until: OccDate) -> Vec<Occ> {
        let occ_day = occ.start.date_naive();
        let start_day = occ_day + chrono::Duration::days(1);
        let end_day = until.date_naive();
        if occ_day > end_day {
            return vec![]
        }

        let mut occs = Vec::<Occ>::new();
        for day in sched::DayFilterDaysIter::new(self.sched, start_day) {
            occs.push(self.for_day(day));
            if day > end_day { break }
        }
        occs
    }

    fn generate_first(&self, now: OccDate) -> Option<Occ> {
        let start_day = self.sched.initial_day;
        let today = now.date_naive();
        for day in sched::DayFilterDaysIter::new(self.sched, start_day) {
            if day >= today { return Some(self.for_day(day)) }
        }
        None
    }
}

pub struct ProgressTaskOccGen<'a> {
    pub sched: &'a ProgressTaskSched,
}

impl OccGen for ProgressTaskOccGen<'_> {
    fn generate_after(&self, occ: &Occ, until: OccDate) -> Vec<Occ> {
        let start_day = occ.end.date_naive();
        let end_day = until.date_naive();
        if occ.end.date_naive() > end_day {
            return vec![]
        }

        let mut occs = Vec::<Occ>::new();
        for (occ_start_day, occ_end_day) in
            sched::ProgressTaskPeriodsIter::new(self.sched, start_day)
        {
            occs.push(new_occ(
                day_to_occ_date(occ_start_day),
                day_to_occ_date(occ_end_day)));
            if occ_end_day > end_day { break }
        }
        occs
    }

    fn generate_first(&self, now: OccDate) -> Option<Occ> {
        sched::ProgressTaskPeriodsIter::new(self.sched, now.date_naive())
            .next()
            .map(|(start_day, end_day)| {
                new_occ(day_to_occ_date(start_day), day_to_occ_date(end_day))
            })
    }
}

pub struct DeadlineTaskOccGen<'a> {
    pub sched: &'a DeadlineTaskSched,
}

impl OccGen for DeadlineTaskOccGen<'_> {
    fn generate_after(&self, occ: &Occ, until: OccDate) -> Vec<Occ> {
        let mut start = occ.end;
        let mut occs = Vec::<Occ>::new();
        while start <= until {
            let end = start + self.sched.duration;
            occs.push(new_occ(start, end));
            start = end;
        }
        occs
    }

    fn generate_first(&self, now: OccDate) -> Option<Occ> {
        Some(new_occ(now, now + self.sched.duration))
    }
}
