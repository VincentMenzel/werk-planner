use crate::model::{Shift, Worker, WorkerId};
use crate::rule::{Rule, RulePriority, RuleScope, RuleViolation, ScheduleContext};
use chrono::{Datelike, NaiveDate, Weekday};
use std::collections::{BTreeSet, HashSet};

#[derive(Debug, Clone)]
pub struct DayOffRule {
    pub id: String,
    pub name: String,
    pub priority: RulePriority,
    pub scope: RuleScope,
    pub forbidden_weekdays: HashSet<Weekday>,
    pub forbidden_dates: BTreeSet<NaiveDate>,
}

impl DayOffRule {
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
            forbidden_weekdays: HashSet::new(),
            forbidden_dates: BTreeSet::new(),
        }
    }

    pub fn for_weekday(
        id: impl Into<String>,
        name: impl Into<String>,
        weekday: Weekday,
        priority: RulePriority,
        scope: RuleScope,
    ) -> Self {
        let mut r = Self::new(id, name, priority, scope);
        r.forbidden_weekdays.insert(weekday);
        r
    }

    pub fn for_date(
        id: impl Into<String>,
        name: impl Into<String>,
        date: NaiveDate,
        priority: RulePriority,
        scope: RuleScope,
    ) -> Self {
        let mut r = Self::new(id, name, priority, scope);
        r.forbidden_dates.insert(date);
        r
    }

    pub fn with_weekday(mut self, weekday: Weekday) -> Self {
        self.forbidden_weekdays.insert(weekday);
        self
    }

    pub fn with_date(mut self, date: NaiveDate) -> Self {
        self.forbidden_dates.insert(date);
        self
    }

    fn is_forbidden(&self, date: NaiveDate) -> bool {
        self.forbidden_dates.contains(&date) || self.forbidden_weekdays.contains(&date.weekday())
    }
}

impl Rule for DayOffRule {
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
        !self.is_forbidden(shift.date)
    }

    fn evaluate(&self, context: &ScheduleContext) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let target_workers: Vec<&WorkerId> = match &self.scope {
            RuleScope::Global => context.workers.keys().collect(),
            RuleScope::Worker(id) => vec![id],
        };

        for worker_id in target_workers {
            for shift in context.worker_shifts(worker_id) {
                if self.is_forbidden(shift.date) {
                    violations.push(RuleViolation {
                        rule_id: self.id.clone(),
                        rule_name: self.name.clone(),
                        priority: self.priority,
                        worker_id: Some(worker_id.clone()),
                        shift_id: Some(shift.id.clone()),
                        message: format!(
                            "Worker '{}' assigned to shift '{}' on forbidden date/weekday {:?}",
                            worker_id,
                            shift.id,
                            shift.date.weekday()
                        ),
                    });
                }
            }
        }

        violations
    }
}
