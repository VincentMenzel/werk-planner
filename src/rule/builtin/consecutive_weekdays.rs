use crate::model::{Shift, Worker, WorkerId};
use crate::rule::{Rule, RulePriority, RuleScope, RuleViolation, ScheduleContext};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct NoConsecutiveWeekdayRule {
    pub id: String,
    pub name: String,
    pub weekday: Weekday,
    pub max_consecutive: usize,
    pub priority: RulePriority,
    pub scope: RuleScope,
}

impl NoConsecutiveWeekdayRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        weekday: Weekday,
        priority: RulePriority,
        scope: RuleScope,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            weekday,
            max_consecutive: 1,
            priority,
            scope,
        }
    }

    pub fn with_max_consecutive(mut self, max: usize) -> Self {
        self.max_consecutive = max;
        self
    }

    fn streak_including(
        &self,
        existing_dates: &BTreeSet<NaiveDate>,
        candidate_date: NaiveDate,
    ) -> usize {
        let mut backward_count = 0;
        let mut cur = candidate_date - Duration::days(7);
        while existing_dates.contains(&cur) {
            backward_count += 1;
            cur -= Duration::days(7);
        }

        let mut forward_count = 0;
        cur = candidate_date + Duration::days(7);
        while existing_dates.contains(&cur) {
            forward_count += 1;
            cur += Duration::days(7);
        }

        backward_count + 1 + forward_count
    }
}

impl Rule for NoConsecutiveWeekdayRule {
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

        if shift.date.weekday() != self.weekday {
            return true;
        }

        if let Some(dates) = context.worker_dates(&worker.id) {
            if dates.contains(&shift.date) {
                return true;
            }
            let streak = self.streak_including(dates, shift.date);
            streak <= self.max_consecutive
        } else {
            1 <= self.max_consecutive
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
                let weekday_dates: Vec<NaiveDate> = dates
                    .iter()
                    .copied()
                    .filter(|d| d.weekday() == self.weekday)
                    .collect();

                let mut current_streak = 0;
                let mut prev_date: Option<NaiveDate> = None;

                for date in weekday_dates {
                    if let Some(prev) = prev_date {
                        if date == prev + Duration::days(7) {
                            current_streak += 1;
                        } else {
                            current_streak = 1;
                        }
                    } else {
                        current_streak = 1;
                    }

                    if current_streak > self.max_consecutive {
                        violations.push(RuleViolation {
                            rule_id: self.id.clone(),
                            rule_name: self.name.clone(),
                            priority: self.priority,
                            worker_id: Some(worker_id.clone()),
                            shift_id: None,
                            message: format!(
                                "Worker '{}' worked {} consecutive {:?}s on {}",
                                worker_id, current_streak, self.weekday, date
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
