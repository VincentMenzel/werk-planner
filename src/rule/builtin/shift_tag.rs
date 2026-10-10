use crate::model::{Shift, Worker, WorkerId};
use crate::rule::{Rule, RulePriority, RuleScope, RuleViolation, ScheduleContext};

#[derive(Debug, Clone)]
pub struct ShiftTagForbiddenRule {
    pub id: String,
    pub name: String,
    pub forbidden_tag: String,
    pub priority: RulePriority,
    pub scope: RuleScope,
}

impl ShiftTagForbiddenRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        forbidden_tag: impl Into<String>,
        priority: RulePriority,
        scope: RuleScope,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            forbidden_tag: forbidden_tag.into(),
            priority,
            scope,
        }
    }
}

impl Rule for ShiftTagForbiddenRule {
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
        _context: &ScheduleContext,
    ) -> bool {
        if !self.scope.applies_to_worker(&worker.id) {
            return true;
        }
        !shift.tags.contains(&self.forbidden_tag)
    }

    fn evaluate(&self, context: &ScheduleContext) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let target_workers: Vec<&WorkerId> = match &self.scope {
            RuleScope::Global => context.workers.keys().collect(),
            RuleScope::Worker(id) => vec![id],
        };

        for worker_id in target_workers {
            for shift in context.worker_shifts(worker_id) {
                if shift.tags.contains(&self.forbidden_tag) {
                    violations.push(RuleViolation {
                        rule_id: self.id.clone(),
                        rule_name: self.name.clone(),
                        priority: self.priority,
                        worker_id: Some(worker_id.clone()),
                        shift_id: Some(shift.id.clone()),
                        message: format!(
                            "Worker '{}' assigned to shift '{}' with forbidden tag '{}'",
                            worker_id, shift.id, self.forbidden_tag
                        ),
                    });
                }
            }
        }

        violations
    }
}
