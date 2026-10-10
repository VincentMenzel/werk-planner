use crate::model::{ShiftId, WorkerId};
use crate::rule::{RulePriority, RuleViolation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolveStatus {
    Success,
    GoalUnreached,
    NoWorkers,
    NoShifts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSatisfaction {
    pub rule_id: String,
    pub rule_name: String,
    pub priority: RulePriority,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolveResult {
    pub status: SolveStatus,
    pub schedule: HashMap<ShiftId, Vec<WorkerId>>,
    pub total_required_slots: usize,
    pub covered_slots: usize,
    pub coverage_ratio: f64,
    pub target_coverage_ratio: f64,
    pub target_slots: usize,
    pub satisfied_rules: Vec<RuleSatisfaction>,
    pub dropped_rules: Vec<RuleViolation>,
    pub soft_priority_score: u64,
    pub total_soft_priority_possible: u64,
    pub iterations: usize,
}

impl SolveResult {
    pub fn is_success(&self) -> bool {
        self.status == SolveStatus::Success
    }

    pub fn was_rule_dropped(&self, rule_id: &str) -> bool {
        self.dropped_rules.iter().any(|v| v.rule_id == rule_id)
    }

    pub fn is_rule_satisfied(&self, rule_id: &str) -> bool {
        self.satisfied_rules
            .iter()
            .any(|r| r.rule_id == rule_id && r.satisfied)
    }

    pub fn assigned_workers(&self, shift_id: &ShiftId) -> &[WorkerId] {
        self.schedule
            .get(shift_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}
