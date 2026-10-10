use crate::model::{Shift, Worker, WorkerId};
use crate::rule::{Rule, RulePriority, RuleScope, RuleViolation, ScheduleContext};

#[derive(Debug, Clone)]
pub struct NoOverlappingShiftsRule {
    pub id: String,
    pub name: String,
    pub priority: RulePriority,
    pub scope: RuleScope,
}

impl NoOverlappingShiftsRule {
    pub fn new(id: impl Into<String>, name: impl Into<String>, priority: RulePriority) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            priority,
            scope: RuleScope::Global,
        }
    }
}

impl Default for NoOverlappingShiftsRule {
    fn default() -> Self {
        Self::new(
            "no_overlapping_shifts",
            "No Overlapping Shifts",
            RulePriority::Required,
        )
    }
}

impl Rule for NoOverlappingShiftsRule {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> RulePriority {
        self.priority
    }

    fn scope(&self) -> &RuleScope {
        &self.scope
    }

    fn check_assignment_hard(
        &self,
        worker: &Worker,
        shift: &Shift,
        context: &ScheduleContext,
    ) -> bool {
        !context.has_overlap(&worker.id, shift)
    }

    fn evaluate(&self, context: &ScheduleContext) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let target_workers: Vec<&WorkerId> = match &self.scope {
            RuleScope::Global => context.workers.keys().collect(),
            RuleScope::Worker(id) => vec![id],
        };

        for worker_id in target_workers {
            let shifts = context.worker_shifts(worker_id);
            for i in 0..shifts.len() {
                for j in (i + 1)..shifts.len() {
                    let s1 = shifts[i];
                    let s2 = shifts[j];
                    if s1.overlaps_with(s2) {
                        violations.push(RuleViolation {
                            rule_id: self.id.clone(),
                            rule_name: self.name.clone(),
                            priority: self.priority,
                            worker_id: Some(worker_id.clone()),
                            shift_id: Some(s2.id.clone()),
                            message: format!(
                                "Worker '{}' assigned to overlapping shifts '{}' and '{}' on {}",
                                worker_id, s1.id, s2.id, s1.date
                            ),
                        });
                    }
                }
            }
        }

        violations
    }
}
