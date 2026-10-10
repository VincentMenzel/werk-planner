use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ShiftId(pub String);

impl From<&str> for ShiftId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ShiftId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for ShiftId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shift {
    pub id: ShiftId,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub required_workers: usize,
    pub tags: BTreeSet<String>,
}

impl Shift {
    pub fn new(
        id: impl Into<ShiftId>,
        date: NaiveDate,
        start_time: NaiveTime,
        end_time: NaiveTime,
    ) -> Self {
        Self {
            id: id.into(),
            date,
            start_time,
            end_time,
            required_workers: 1,
            tags: BTreeSet::new(),
        }
    }

    pub fn with_required_workers(mut self, required_workers: usize) -> Self {
        self.required_workers = required_workers;
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for tag in tags {
            self.tags.insert(tag.into());
        }
        self
    }

    pub fn duration_hours(&self) -> f64 {
        let duration = self.end_time - self.start_time;
        duration.num_minutes() as f64 / 60.0
    }

    pub fn overlaps_with(&self, other: &Shift) -> bool {
        if self.date != other.date {
            return false;
        }
        self.start_time < other.end_time && other.start_time < self.end_time
    }
}
