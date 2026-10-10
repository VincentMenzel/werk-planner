use crate::model::{Shift, Worker, WorkerId};
use crate::rule::{Rule, RulePriority, RuleScope, RuleViolation, ScheduleContext};
use chrono::{Duration, NaiveDate};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct MaxConsecutiveDaysRule {
    pub id: String,
    pub name: String,
    pub max_consecutive_days: usize,
    pub priority: RulePriority,
    pub scope: RuleScope,
}

impl MaxConsecutiveDaysRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        max_consecutive_days: usize,
        priority: RulePriority,
        scope: RuleScope,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            max_consecutive_days,
            priority,
            scope,
        }
    }

    fn streak_including(
        &self,
        existing_dates: &BTreeSet<NaiveDate>,
        candidate_date: NaiveDate,
    ) -> usize {
        let mut backward_count = 0;
        let mut cur = candidate_date - Duration::days(1);
        while existing_dates.contains(&cur) {
            backward_count += 1;
            cur -= Duration::days(1);
        }

        let mut forward_count = 0;
        cur = candidate_date + Duration::days(1);
        while existing_dates.contains(&cur) {
            forward_count += 1;
            cur += Duration::days(1);
        }

        backward_count + 1 + forward_count
    }
}

impl Rule for MaxConsecutiveDaysRule {
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

        if let Some(dates) = context.worker_dates(&worker.id) {
            if dates.contains(&shift.date) {
                // Already scheduled on this day, adding another shift on same day doesn't extend streak
                return true;
            }
            let streak = self.streak_including(dates, shift.date);
            streak <= self.max_consecutive_days
        } else {
            1 <= self.max_consecutive_days
        }
    }

    fn evaluate(&self, context: &ScheduleContext) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let target_workers: Vec<&WorkerId> = match &self.scope {
            RuleScope::Global => context.workers.keys().collect(),
            RuleScope::Worker(id) => vec![id],
        };

        for worker_id in target_workers {
            if let Some(dates) = context.worker_dates(worker_id) {
                let mut current_streak = 0;
                let mut prev_date: Option<NaiveDate> = None;

                for &date in dates {
                    if let Some(prev) = prev_date {
                        if date == prev + Duration::days(1) {
                            current_streak += 1;
                        } else {
                            current_streak = 1;
                        }
                    } else {
                        current_streak = 1;
                    }

                    if current_streak > self.max_consecutive_days {
                        violations.push(RuleViolation {
                            rule_id: self.id.clone(),
                            rule_name: self.name.clone(),
                            priority: self.priority,
                            worker_id: Some(worker_id.clone()),
                            shift_id: None,
                            message: format!(
                                "Worker '{}' exceeded max consecutive days limit ({}) on {}",
                                worker_id, self.max_consecutive_days, date
                            ),
                        });
                    }

                    prev_date = Some(date);
                }
            }
        }

        violations
    }
}
