use crate::model::{Shift, Worker, WorkerId};
use crate::rule::{Rule, RulePriority, RuleScope, RuleViolation, ScheduleContext};

#[derive(Debug, Clone)]
pub struct MinMaxHoursRule {
    pub id: String,
    pub name: String,
    pub priority: RulePriority,
    pub scope: RuleScope,
    pub custom_min: Option<f64>,
    pub custom_max: Option<f64>,
}

impl MinMaxHoursRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        priority: RulePriority,
        scope: RuleScope,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            priority,
            scope,
            custom_min: None,
            custom_max: None,
        }
    }

    pub fn with_custom_bounds(mut self, min: f64, max: f64) -> Self {
        self.custom_min = Some(min);
        self.custom_max = Some(max);
        self
    }

    fn worker_bounds(&self, worker: &Worker) -> (f64, f64) {
        let min = self.custom_min.unwrap_or(worker.min_hours_per_week);
        let max = self.custom_max.unwrap_or(worker.max_hours_per_week);
        (min, max)
    }
}

impl Rule for MinMaxHoursRule {
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
        if !self.scope.applies_to_worker(&worker.id) {
            return true;
        }

        let (_, max_hours) = self.worker_bounds(worker);
        let current_hours = context.worker_hours(&worker.id);
        (current_hours + shift.duration_hours()) <= max_hours + 1e-6
    }

    fn evaluate(&self, context: &ScheduleContext) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let target_workers: Vec<&WorkerId> = match &self.scope {
            RuleScope::Global => context.workers.keys().collect(),
            RuleScope::Worker(id) => vec![id],
        };

        for worker_id in target_workers {
            if let Some(worker) = context.workers.get(worker_id) {
                let (min_hours, max_hours) = self.worker_bounds(worker);
                let hours = context.worker_hours(worker_id);

                if hours < min_hours - 1e-6 {
                    violations.push(RuleViolation {
                        rule_id: self.id.clone(),
                        rule_name: self.name.clone(),
                        priority: self.priority,
                        worker_id: Some(worker_id.clone()),
                        shift_id: None,
                        message: format!(
                            "Worker '{}' worked {:.1} hours, below minimum {:.1} hours",
                            worker_id, hours, min_hours
                        ),
                    });
                }

                if hours > max_hours + 1e-6 {
                    violations.push(RuleViolation {
                        rule_id: self.id.clone(),
                        rule_name: self.name.clone(),
                        priority: self.priority,
                        worker_id: Some(worker_id.clone()),
                        shift_id: None,
                        message: format!(
                            "Worker '{}' worked {:.1} hours, exceeding maximum {:.1} hours",
                            worker_id, hours, max_hours
                        ),
                    });
                }
            }
        }

        violations
    }
}
