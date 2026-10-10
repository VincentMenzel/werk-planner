pub mod builtin;
pub mod context;

pub use context::ScheduleContext;

use crate::model::{Shift, ShiftId, Worker, WorkerId};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleScope {
    Global,
    Worker(WorkerId),
}

impl RuleScope {
    pub fn applies_to_worker(&self, worker_id: &WorkerId) -> bool {
        match self {
            Self::Global => true,
            Self::Worker(id) => id == worker_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RulePriority {
    Soft(u32),
    Required,
}

impl RulePriority {
    pub fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }

    pub fn priority_value(&self) -> u32 {
        match self {
            Self::Required => u32::MAX,
            Self::Soft(p) => *p,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_id: String,
    pub rule_name: String,
    pub priority: RulePriority,
    pub worker_id: Option<WorkerId>,
    pub shift_id: Option<ShiftId>,
    pub message: String,
}

pub trait Rule: Debug + Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn priority(&self) -> RulePriority;
    fn scope(&self) -> &RuleScope;
    fn evaluate(&self, context: &ScheduleContext) -> Vec<RuleViolation>;

    /// Hard constraint check used during search pruning.
    /// Only hard (Required) rules should prune the search tree.
    fn can_assign(&self, worker: &Worker, shift: &Shift, context: &ScheduleContext) -> bool {
        if !self.priority().is_required() {
            return true;
        }
        self.check_assignment_hard(worker, shift, context)
    }

    fn check_assignment_hard(
        &self,
        _worker: &Worker,
        _shift: &Shift,
        _context: &ScheduleContext,
    ) -> bool {
        true
    }
}
