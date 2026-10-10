use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorkerId(pub String);

impl From<&str> for WorkerId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for WorkerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for WorkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Worker {
    pub id: WorkerId,
    pub name: String,
    pub min_hours_per_week: f64,
    pub max_hours_per_week: f64,
}

impl Worker {
    pub fn new(id: impl Into<WorkerId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            min_hours_per_week: 0.0,
            max_hours_per_week: 40.0,
        }
    }

    pub fn with_hours(mut self, min: f64, max: f64) -> Self {
        self.min_hours_per_week = min;
        self.max_hours_per_week = max;
        self
    }
}
