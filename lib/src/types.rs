use core::time::Duration;
use serde::{Deserialize, Serialize};

fn opt_duration_to_chrono(duration: &Option<Duration>) -> chrono::Duration {
    chrono::Duration::from_std(duration.unwrap_or(Duration::ZERO))
        .unwrap_or(chrono::Duration::max_value())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize,
         strum::AsRefStr, strum::EnumString)]
pub enum ItemType {
    /// Occurrences are fixed points in time according to the schedules.
    Event,
    /// Occurrences may vary according to when task completion is logged.
    Task,
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

/// Recurring task with the goal of reaching the target completion amount
/// averaged over completion periods.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ProgressTaskSched {
    pub duration: ProgressTaskDuration,
}

/// Task with deadline based on the previous completion.
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
    pub id: Option<String>,
    pub type_: ItemType,
    /// Used for configuring groups of items.
    pub category: Option<String>,
    pub name: String,
    pub desc: Option<String>,
    pub sched: Sched,
}

pub type OccDate = chrono::DateTime<chrono::offset::Utc>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Occ {
    pub id: Option<String>,
    /// Start of the occurrence period.
    pub start: OccDate,
    /// End of the occurrence period.
    pub end: OccDate,
    /// Any non-zero value counts as 'completed' for tasks without a configured
    /// target completion amount.
    pub task_completion_progress: u32,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ConfigId {
    // in inheritance order, parent first
    All,
    Type(ItemType),
    Category(String),
    Item { id: String },
    Occ { id: String },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
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

    pub fn default() -> TaskCompletionConfig {
        TaskCompletionConfig {
            total: None,
            unit: None,
            excess_past: None,
            excess_future: None,
        }
    }

    pub fn excess_past_chrono(&self) -> chrono::Duration {
        opt_duration_to_chrono(&self.excess_past)
    }

    pub fn excess_future_chrono(&self) -> chrono::Duration {
        opt_duration_to_chrono(&self.excess_future)
    }

}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub id: Option<ConfigId>,
    /// Whether the item is being tracked.
    pub active: Option<bool>,
    /// How long before an occurrence (event's start or task's deadline) to show
    /// alerts/notifications for it.  For ProgressTask schedules, the
    /// occurrence start is used instead.
    pub occ_alert: Option<Duration>,
    /// Applies to ProgressTask schedules.
    pub task_completion_conf: TaskCompletionConfig,
}

impl Config {

    pub fn default() -> Config {
        Config {
            id: None,
            active: None,
            occ_alert: None,
            task_completion_conf: TaskCompletionConfig::default(),
        }
    }

    pub fn occ_alert_chrono(&self) -> chrono::Duration {
        opt_duration_to_chrono(&self.occ_alert)
    }

}
