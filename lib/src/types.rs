use core::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize,
         strum::AsRefStr, strum::EnumString)]
pub enum ItemType {
    /// Occurrences are fixed points in time according to the schedules.
    Event,
    /// Occurrences may vary according to when task completion is logged.
    Task,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Item {
    pub id: Option<String>,
    pub type_: ItemType,
    /// Used for configuring groups of items.
    pub category: Option<String>,
    pub desc: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum DayFilterType {
    Include,
    Exclude,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum DayFilterSel {
    DaysBetween(u8),
    WeeksBetween(u8),
    MonthsBetween(u8),
    YearsBetween(u8),
    Dow(Vec<u8>),
    Dom(Vec<u8>),
    Doy(Vec<u16>),
    Wom(Vec<u8>),
    Woy(Vec<u8>),
    Moy(Vec<u8>),
    Y(Vec<i32>),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct DayFilter {
    pub type_: DayFilterType,
    pub sel: DayFilterSel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum AvgCompletionTaskDuration {
    Days {
        num: u8,
    },
    Weeks {
        num: u8,
        start_day: chrono::Weekday,
    },
    Months {
        num: u8,
        start_day: u8,
    },
    Years {
        num: u8,
        start_month: chrono::Month,
        start_dom: u8,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum SchedOcc {
    Event {
        /// Needed to make days deterministic.
        initial_day: chrono::NaiveDate,
        /// Describes the days the event occurs on.
        days: Vec<DayFilter>,
        /// Time of day the event occurs at, for any timezone.
        time: Option<chrono::NaiveTime>,
    },
    /// Recurring task with the goal of reaching the target completion amount
    /// averaged over completion periods.
    AvgCompletionTask {
        duration: AvgCompletionTaskDuration,
    },
    /// Task with deadline based on the previous completion.
    DeadlineTask {
        /// Time from completing the task to the next deadline.
        duration: Duration,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Sched {
    pub id: Option<String>,
    pub active: bool,
    occ: SchedOcc,
    desc: Option<String>,
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
    /// Excess completion can count towards incomplete tasks up to this far in
    /// the past.
    pub excess_past: Option<Duration>,
    /// Excess completion can count towards incomplete tasks up to this far in
    /// the future.
    pub excess_future: Option<Duration>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub id: Option<ConfigId>,
    /// Whether the item is being tracked.
    pub active: Option<bool>,
    /// How long before an occurrence (event's start or task's deadline) to show
    /// alerts/notifications for it.
    pub occ_alert: Option<Duration>,
    /// Applies to AvgCompletionTask schedules.
    pub task_completion_conf: TaskCompletionConfig,
}
