use core::time::Duration;
use serde::{Deserialize, Serialize};

fn opt_duration_to_chrono(duration: &Option<Duration>) -> chrono::Duration {
    chrono::Duration::from_std(duration.unwrap_or(Duration::ZERO))
        .unwrap_or(chrono::Duration::max_value())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize,
         strum::AsRefStr, strum::EnumString)]
pub enum ItemType {
    /// Occurrences are fixed points in time according to the schedule.
    Event,
    /// Occurrences cover fixed completion periods, with the goal of reaching
    /// the target completion amount averaged over these periods.
    ProgressTask,
    /// Occurrences have a deadline based on the previous completion.
    DeadlineTask,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum DayFilter {
    Day {
        days_apart: u32,
    },
    Dow {
        day: chrono::Weekday,
        weeks_apart: u32,
    },
    Dows {
        days: Vec<chrono::Weekday>,
    },
    Dom {
        /// starting from 1, including the last day once if any days don't exist
        days: Vec<u8>,
        months_apart: u32,
    },
    Wom {
        dow: chrono::Weekday,
        /// starting from 1, meaning the first occurrence of each of the
        /// specified days of the week
        weeks: Vec<u8>,
        months_apart: u32,
    },
    Doy {
        /// day starting from 1, using the last day instead if doesn't exist
        dom: u8,
        month: chrono::Month,
        years_apart: u32,
    },
    Date {
        dom: u8,
        month: chrono::Month,
        year: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ProgressTaskDuration {
    Days {
        num: u8,
    },
    Weeks {
        num: u8,
        start_day: chrono::Weekday,
    },
    Months {
        num: u8,
        /// starting from 1
        start_day: u8,
    },
    Years {
        num: u8,
        start_month: chrono::Month,
        /// starting from 1
        start_dom: u8,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct EventSched {
    /// Needed to make days deterministic.
    pub initial_day: chrono::NaiveDate,
    /// Describes the days the event occurs on.
    pub days: DayFilter,
    /// Time of day the event occurs at, for any timezone.
    pub time: Option<chrono::NaiveTime>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ProgressTaskSched {
    pub duration: ProgressTaskDuration,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct DeadlineTaskSched {
    /// Time from completing the task to the next deadline.
    pub duration: Duration,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum Sched {
    Event(EventSched),
    ProgressTask(ProgressTaskSched),
    DeadlineTask(DeadlineTaskSched),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Item {
    pub type_: ItemType,
    /// Whether the item is being tracked.
    pub active: bool,
    /// Used for configuring groups of items.
    pub category: Option<String>,
    pub name: String,
    pub desc: Option<String>,
    pub sched: Sched,
}

pub type OccDate = chrono::DateTime<chrono::offset::Utc>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Occ {
    /// Whether the occurrence is being tracked.
    pub active: bool,
    /// Start of the occurrence period.
    pub start: OccDate,
    /// End of the occurrence period.
    pub end: OccDate,
    /// Any non-zero value counts as 'completed' for tasks without a configured
    /// target completion amount.
    pub task_completion_progress: u32,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TaskCompletionConfig {
    /// Target completion amount.
    pub total: Option<u32>,
    /// Display unit for completion value.
    pub unit: Option<String>,
    /// Excess completion from other occurrences can count towards this
    /// occurrence up to this far in the past.
    pub excess_past: Option<Duration>,
    /// Excess completion from other occurrences can count towards this
    /// occurrence up to this far in the future.
    pub excess_future: Option<Duration>,
}

impl TaskCompletionConfig {
    pub fn excess_past_chrono(&self) -> chrono::Duration {
        opt_duration_to_chrono(&self.excess_past)
    }

    pub fn excess_future_chrono(&self) -> chrono::Duration {
        opt_duration_to_chrono(&self.excess_future)
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// How long before an occurrence (event's start or task's deadline) to show
    /// alerts/notifications for it.  For progress tasks, the occurrence start
    /// is used instead.
    pub occ_alert: Option<Duration>,
    /// Applies to progress tasks.
    pub task_completion_conf: TaskCompletionConfig,
}

impl Config {
    pub fn occ_alert_chrono(&self) -> chrono::Duration {
        opt_duration_to_chrono(&self.occ_alert)
    }
}
