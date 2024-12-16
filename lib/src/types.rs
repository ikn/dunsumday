//! Core library types.
//!
//! [Items](Item) contains [occurrences](Occ).  Both can be
//! [configured](Config).

use core::time::Duration;
use serde::{Deserialize, Serialize};

/// Convert optional duration to chrono duration, defaulting to zero.
fn opt_duration_to_chrono(duration: &Option<Duration>) -> chrono::TimeDelta {
    chrono::TimeDelta::from_std(duration.unwrap_or(Duration::ZERO))
        .unwrap_or(chrono::TimeDelta::MAX)
}

/// Allowed types for [items](Item).
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

/// Describes the days an event occurs on.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum DayFilter {
    /// Once every `days_apart` days.
    Day {
        days_apart: u32,
    },
    /// Once every `weeks_apart` weeks, always falling on the same `day`.
    Dow {
        day: chrono::Weekday,
        weeks_apart: u32,
    },
    /// On every one of the specified `days` of the week.
    Dows {
        days: Vec<chrono::Weekday>,
    },
    /// On every one of the specified `days` of the month, every `months_apart`
    /// months.
    Dom {
        /// Starting from 1, including the last day once if any days don't
        /// exist.
        days: Vec<u8>,
        months_apart: u32,
    },
    /// On every one of the specified `weeks` occurrences in the month, of the
    /// specified day of the week `dow`, every `months_apart` months.  For
    /// example, "every 2nd and 3rd Tuesday of every 6th month".
    Wom {
        dow: chrono::Weekday,
        /// Starting from 1, meaning the first occurrence of the specified day
        /// of the week.
        weeks: Vec<u8>,
        months_apart: u32,
    },
    /// On the specific day of the month `dom`, and `month`, every `years_apart`
    /// years.
    Doy {
        /// Day starting from 1, using the last day instead if doesn't exist.
        dom: u8,
        month: chrono::Month,
        years_apart: u32,
    },
    /// On the specific day of the month `dom`, `month`, and `year`.
    Date {
        dom: u8,
        month: chrono::Month,
        /// A `chrono` year, i.e. negative values are BCE.
        year: i32,
    },
}


/// Schedule for events.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct EventSched {
    /// The first date the event occurs on.
    pub initial_day: chrono::NaiveDate,
    /// Describes the days the event occurs on.
    pub days: DayFilter,
    /// Time of day the event occurs at, for any timezone.
    pub time: Option<chrono::NaiveTime>,
}

/// Schedule for progress tasks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ProgressTaskSched {
    /// Duration of `num` days.
    Days {
        num: u8,
    },
    /// Duration of `num` weeks, always starting on day of the week `start_day`.
    Weeks {
        num: u8,
        start_day: chrono::Weekday,
    },
    /// Duration of `num` months, always starting on day of the month
    /// `start_day`.
    Months {
        num: u8,
        /// Starting from 1.
        start_day: u8,
    },
    /// Duration of `num` years, always starting on day of the month `start_dom`
    /// and month `start_month`.
    Years {
        num: u8,
        start_month: chrono::Month,
        /// Starting from 1.
        start_dom: u8,
    },
}

/// Schedule for deadline tasks.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct DeadlineTaskSched {
    /// Time from completing the task to the next deadline.
    pub duration: Duration,
}

/// Schedule for an item.
///
/// Should match the [item type](ItemType), but there is nothing to enforce
/// this.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum Sched {
    Event(EventSched),
    ProgressTask(ProgressTaskSched),
    DeadlineTask(DeadlineTaskSched),
}

/// An event or task.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Item {
    pub type_: ItemType,
    /// Whether the item is being tracked.
    pub active: bool,
    /// Used for [configuring](Config) groups of items.
    pub category: Option<String>,
    pub name: String,
    pub desc: Option<String>,
    pub sched: Sched,
}

/// Type of date used for occurrences.
pub type OccDate = chrono::DateTime<chrono::offset::Utc>;

/// Occurrence of an item.
///
/// This is the period of time across which a task is to be completed, or the
/// point in time for an instance of an event.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Occ {
    /// Whether the occurrence is being tracked.
    pub active: bool,
    /// Start of the occurrence period.
    pub start: OccDate,
    /// End of the occurrence period.
    pub end: OccDate,
    /// For tasks, this is used to track progress.  Any non-zero value counts as
    /// 'completed' for tasks without a [configured](TaskCompletionConfig)
    /// target completion amount.
    pub task_completion_progress: u32,
}

/// Configuration that applies to progress tasks.
///
/// Also see [Config].
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
    /// `excess_past` as a chrono duration.
    pub fn excess_past_chrono(&self) -> chrono::TimeDelta {
        opt_duration_to_chrono(&self.excess_past)
    }

    /// `excess_future` as a chrono duration.
    pub fn excess_future_chrono(&self) -> chrono::TimeDelta {
        opt_duration_to_chrono(&self.excess_future)
    }
}

/// Configuration for occurrences.
///
/// Via [ConfigId](crate::db::ConfigId), this can be applied to different
/// scopes, such as a specific occurrences, all occurrences for an item, or all
/// occurrences for all items of a specific type.
///
/// All values are optional.  Config applied at a more specific scope takes
/// precedence---for example, config applied to an occurrence take precedence
/// over config applied to an item.  For each field, the value is taken from the
/// config with the highest precedence which has a value.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// How long before an occurrence to show alerts/notifications for it.  For
    /// events and progress tasks, this is the start; for deadline tasks, this
    /// is the deadline (end).
    pub occ_alert: Option<Duration>,
    /// Applies to progress tasks.
    pub task_completion_conf: TaskCompletionConfig,
}

impl Config {
    /// `occ_alert` as a chrono duration.
    pub fn occ_alert_chrono(&self) -> chrono::TimeDelta {
        opt_duration_to_chrono(&self.occ_alert)
    }
}
