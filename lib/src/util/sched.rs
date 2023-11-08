use std::cmp::min;
use std::collections::{BTreeSet, HashSet};
use std::iter::Iterator;
use chrono::{Datelike, NaiveDate, naive};
use crate::types::{AvgCompletionTaskDuration::*, AvgCompletionTaskSched,
                   DayFilter, EventSched};

fn year_of_date(date: NaiveDate) -> i32 {
    let (ce, year) = date.year_ce();
    if ce { year as i32 } else { -(year as i32) }
}

fn days_in_month(date: NaiveDate) -> u8 {
    let year = year_of_date(date);
    let month = date.month0() + 1;
    let start_date = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or(NaiveDate::MAX);
    let end_date = if date.month0() == 11 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap_or(NaiveDate::MAX)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap_or(NaiveDate::MAX)
    };
    end_date.signed_duration_since(start_date).num_days() as u8
}

fn of_year(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(NaiveDate::MAX)
}

fn with_dom_saturating(date: NaiveDate, dom: u8) -> NaiveDate {
    date.with_day(min(dom, days_in_month(date)).into())
        .unwrap_or(NaiveDate::MAX)
}

fn with_moy_dom_saturating(date: NaiveDate, moy: chrono::Month, dom: u8)
-> NaiveDate {
    let with_moy = date.with_month(moy.number_from_month())
        .unwrap_or(NaiveDate::MAX);
    with_dom_saturating(with_moy, dom)
}

fn forwards_to_dow(date: NaiveDate, dow: chrono::Weekday) -> NaiveDate {
    let dow_diff = dow.number_from_monday() -
                   date.weekday().number_from_monday();
    if dow_diff == 0 {
        date
    } else {
        date + naive::Days::new((dow_diff % 7).into())
    }
}

fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    date
        .checked_add_months(chrono::Months::new(months))
        .unwrap_or(NaiveDate::MAX)
}

pub struct DayFilterDaysIter<'a> {
    sched: &'a EventSched,
    day: NaiveDate,
    dows_days: HashSet<chrono::Weekday>,
    dom_days: BTreeSet<u8>,
    wom_weeks: HashSet<u8>,
}

impl DayFilterDaysIter<'_> {
    /// may include the start day
    pub fn new(sched: &EventSched, start_day: NaiveDate) -> DayFilterDaysIter {
        let dows_days = match &sched.days {
            DayFilter::Dows { days } => {
                HashSet::from_iter(days.iter().cloned())
            },
            _ => HashSet::new(),
        };

        let dom_days = match &sched.days {
            DayFilter::Dom { days, months_apart } => {
                BTreeSet::from_iter(days.iter().cloned())
            },
            _ => BTreeSet::new()
        };

        let wom_weeks = match &sched.days {
            DayFilter::Wom { dow, weeks, months_apart } => {
                HashSet::from_iter(weeks.iter().cloned())
            },
            _ => HashSet::new(),
        };

        DayFilterDaysIter {
            sched, day: start_day, dows_days, dom_days, wom_weeks }
    }
}

impl Iterator for DayFilterDaysIter<'_> {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        let now = self.day;
        match &self.sched.days {

            DayFilter::Day { days_apart } => {
                self.day = now + naive::Days::new((*days_apart).into());
                Some(now)
            },

            DayFilter::Dow { day: dow, weeks_apart } => {
                let day = forwards_to_dow(now, *dow);
                self.day = day + naive::Days::new(u64::from(*weeks_apart) * 7);
                Some(day)
            },

            DayFilter::Dows { days } => {
                if self.dows_days.is_empty() {
                    return None
                }

                let mut day = now;
                while !self.dows_days.contains(&day.weekday()) {
                    day = day + naive::Days::new(1);
                }
                self.day = day + naive::Days::new(1);
                Some(day)
            },

            DayFilter::Dom { days, months_apart } => {
                if self.dom_days.is_empty() {
                    return None
                }

                let day = match self.dom_days.range(now.day() as u8 ..).next() {
                    Some(dom) => {
                        if now.day() == (*dom).into() {
                            Some(now)
                        } else {
                            let day = with_dom_saturating(now, *dom);
                            if day == now {
                                None
                            } else {
                                Some(day)
                            }
                        }
                    },
                    None => None
                };

                let day = day.unwrap_or_else(|| {
                    let next_month = add_months(
                        with_dom_saturating(now, 1), *months_apart);
                    with_dom_saturating(
                        next_month, *self.dom_days.first().unwrap())
                });

                self.day = day + naive::Days::new(1);
                if self.day.month0() != day.month0() {
                    self.day = add_months(self.day, months_apart - 1);
                }
                Some(day)
            },

            DayFilter::Wom { dow, weeks, months_apart } => {
                if weeks.iter().all(|w| *w > 5) {
                    return None
                }

                let mut day = forwards_to_dow(now, *dow);
                while !self.wom_weeks.contains(&(day.day0() as u8 / 7 + 1)) {
                    day = day + naive::Days::new(7);
                }

                self.day = day + naive::Days::new(7);
                Some(day)
            },

            DayFilter::Doy { dom, month, years_apart } => {
                let this_year = with_moy_dom_saturating(now, *month, *dom);
                let day = if this_year > now {
                    this_year
                } else {
                    let year = year_of_date(now)
                        .saturating_add_unsigned(*years_apart);
                    with_moy_dom_saturating(of_year(year), *month, *dom)
                };

                self.day = of_year(
                    year_of_date(day).saturating_add_unsigned(*years_apart));
                Some(day)
            },

        }
    }
}

/// items are (start_day, end_day) for each occurrence
pub struct AvgCompletionTaskPeriodsIter<'a> {
    sched: &'a AvgCompletionTaskSched,
    day: NaiveDate,
}

impl AvgCompletionTaskPeriodsIter<'_> {
    /// the first item will include the start day
    pub fn new(sched: &AvgCompletionTaskSched, start_day: NaiveDate)
    -> AvgCompletionTaskPeriodsIter {
        AvgCompletionTaskPeriodsIter { sched, day: start_day }
    }
}

impl Iterator for AvgCompletionTaskPeriodsIter<'_> {
    type Item = (NaiveDate, NaiveDate);

    fn next(&mut self) -> Option<Self::Item> {
        let (start, end) = match self.sched.duration {

            Days { num } => {
                (self.day, self.day + naive::Days::new(num.into()))
            },

            Weeks { num, start_day: dow } => {
                let now = self.day;
                let dow_diff = now.weekday().number_from_monday() -
                               dow.number_from_monday();
                let start = if dow_diff == 0 {
                    now
                } else {
                    now - naive::Days::new((dow_diff % 7).into())
                };
                (start, start + naive::Days::new(7 * u64::from(num)))
            },

            Months { num, start_day: dom } => {
                let now = self.day;
                let now_dom = now.day() as u8;
                // move backwards to match start_day
                let start = if now_dom < dom {
                    let month_ago = now
                        .checked_sub_months(chrono::Months::new(1))
                        .unwrap_or(NaiveDate::MIN);
                    with_dom_saturating(month_ago, dom)
                } else {
                    now.with_day(dom.into()).unwrap_or(NaiveDate::MIN)
                };

                let end = add_months(start, num.into());
                if end.day() != dom.into() {
                    let end = with_dom_saturating(end, dom);
                }

                (start, end)
            },

            Years { num, start_month: moy, start_dom: dom } => {
                let now = self.day;
                let start_this_year = with_moy_dom_saturating(now, moy, dom);

                if now <= start_this_year {
                    let next_year = year_of_date(start_this_year) + 1;
                    (start_this_year,
                     with_moy_dom_saturating(of_year(next_year), moy, dom))

                } else {
                    let last_year = year_of_date(start_this_year) - 1;
                    (with_moy_dom_saturating(of_year(last_year), moy, dom),
                     start_this_year)
                }
            },

        };
        self.day = end;
        Some((start, end))
    }
}
