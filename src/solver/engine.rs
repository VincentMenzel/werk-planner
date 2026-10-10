use crate::error::PlannerError;
use crate::model::{CoverageGoal, Shift, ShiftId, Worker, WorkerId};
use crate::rule::{Rule, RulePriority, RuleViolation, ScheduleContext};
use crate::solver::result::{RuleSatisfaction, SolveResult, SolveStatus};
use std::collections::{HashMap, HashSet};

struct Slot {
    shift_id: ShiftId,
    _slot_index: usize,
}

#[derive(Clone)]
struct BestCandidate {
    soft_score: u64,
    covered_slots: usize,
    assignments: HashMap<ShiftId, Vec<WorkerId>>,
    violations: Vec<RuleViolation>,
    satisfied_rules: Vec<RuleSatisfaction>,
}

pub struct Planner {
    workers: Vec<Worker>,
    shifts: Vec<Shift>,
    rules: Vec<Box<dyn Rule>>,
    goal: CoverageGoal,
    max_iterations: usize,
}

impl Planner {
    pub fn new() -> Self {
        Self {
            workers: Vec::new(),
            shifts: Vec::new(),
            rules: Vec::new(),
            goal: CoverageGoal::default(),
            max_iterations: 500_000,
        }
    }

    pub fn add_worker(mut self, worker: Worker) -> Self {
        self.workers.push(worker);
        self
    }

    pub fn with_workers(mut self, workers: Vec<Worker>) -> Self {
        self.workers = workers;
        self
    }

    pub fn add_shift(mut self, shift: Shift) -> Self {
        self.shifts.push(shift);
        self
    }

    pub fn with_shifts(mut self, shifts: Vec<Shift>) -> Self {
        self.shifts = shifts;
        self
    }

    pub fn add_rule<R: Rule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn add_boxed_rule(mut self, rule: Box<dyn Rule>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_rules(mut self, rules: Vec<Box<dyn Rule>>) -> Self {
        self.rules = rules;
        self
    }

    pub fn with_goal(mut self, goal: CoverageGoal) -> Self {
        self.goal = goal;
        self
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    fn validate(&self) -> Result<(), PlannerError> {
        let mut worker_ids = HashSet::new();
        for worker in &self.workers {
            if !worker_ids.insert(&worker.id) {
                return Err(PlannerError::DuplicateWorker(worker.id.0.clone()));
            }
        }

        let mut shift_ids = HashSet::new();
        for shift in &self.shifts {
            if !shift_ids.insert(&shift.id) {
                return Err(PlannerError::DuplicateShift(shift.id.0.clone()));
            }
            if shift.end_time <= shift.start_time {
                return Err(PlannerError::InvalidShiftDuration(
                    shift.start_time.to_string(),
                    shift.end_time.to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn solve(&self) -> Result<SolveResult, PlannerError> {
        self.validate()?;

        if self.shifts.is_empty() {
            return Ok(SolveResult {
                status: SolveStatus::NoShifts,
                schedule: HashMap::new(),
                total_required_slots: 0,
                covered_slots: 0,
                coverage_ratio: 1.0,
                target_coverage_ratio: 1.0,
                target_slots: 0,
                satisfied_rules: Vec::new(),
                dropped_rules: Vec::new(),
                soft_priority_score: 0,
                total_soft_priority_possible: 0,
                iterations: 0,
            });
        }

        let total_required_slots: usize = self.shifts.iter().map(|s| s.required_workers).sum();
        let target_slots = self.goal.required_slots(total_required_slots);
        let target_coverage_ratio = if total_required_slots == 0 {
            1.0
        } else {
            target_slots as f64 / total_required_slots as f64
        };

        if self.workers.is_empty() {
            if target_slots == 0 {
                let mut empty_sched = HashMap::new();
                for s in &self.shifts {
                    empty_sched.insert(s.id.clone(), Vec::new());
                }
                return Ok(SolveResult {
                    status: SolveStatus::Success,
                    schedule: empty_sched,
                    total_required_slots,
                    covered_slots: 0,
                    coverage_ratio: 0.0,
                    target_coverage_ratio,
                    target_slots,
                    satisfied_rules: Vec::new(),
                    dropped_rules: Vec::new(),
                    soft_priority_score: 0,
                    total_soft_priority_possible: 0,
                    iterations: 0,
                });
            } else {
                return Ok(SolveResult {
                    status: SolveStatus::NoWorkers,
                    schedule: HashMap::new(),
                    total_required_slots,
                    covered_slots: 0,
                    coverage_ratio: 0.0,
                    target_coverage_ratio,
                    target_slots,
                    satisfied_rules: Vec::new(),
                    dropped_rules: Vec::new(),
                    soft_priority_score: 0,
                    total_soft_priority_possible: 0,
                    iterations: 0,
                });
            }
        }

        // Build lookup maps
        let workers_map: HashMap<WorkerId, Worker> = self
            .workers
            .iter()
            .cloned()
            .map(|w| (w.id.clone(), w))
            .collect();

        // Sort shifts chronologically
        let mut sorted_shifts = self.shifts.clone();
        sorted_shifts.sort_by_key(|a| (a.date, a.start_time));

        let shifts_map: HashMap<ShiftId, Shift> = sorted_shifts
            .iter()
            .cloned()
            .map(|s| (s.id.clone(), s))
            .collect();

        let shift_order: Vec<ShiftId> = sorted_shifts.iter().map(|s| s.id.clone()).collect();

        // Expand into discrete slots
        let mut slots = Vec::new();
        for shift in &sorted_shifts {
            for slot_idx in 0..shift.required_workers {
                slots.push(Slot {
                    shift_id: shift.id.clone(),
                    _slot_index: slot_idx,
                });
            }
        }

        let total_soft_priority_possible: u64 = self
            .rules
            .iter()
            .filter_map(|r| match r.priority() {
                RulePriority::Soft(p) => Some(p as u64),
                RulePriority::Required => None,
            })
            .sum();

        let mut context = ScheduleContext::new(&workers_map, &shifts_map, &shift_order);
        let mut iterations = 0;
        let mut best: Option<BestCandidate> = None;
        let mut max_covered_ever = 0;

        self.backtrack(
            0,
            0,
            &slots,
            &mut context,
            target_slots,
            total_required_slots,
            total_soft_priority_possible,
            &mut iterations,
            &mut best,
            &mut max_covered_ever,
        );

        if let Some(solution) = best {
            let coverage_ratio = if total_required_slots == 0 {
                1.0
            } else {
                solution.covered_slots as f64 / total_required_slots as f64
            };

            Ok(SolveResult {
                status: SolveStatus::Success,
                schedule: solution.assignments,
                total_required_slots,
                covered_slots: solution.covered_slots,
                coverage_ratio,
                target_coverage_ratio,
                target_slots,
                satisfied_rules: solution.satisfied_rules,
                dropped_rules: solution.violations,
                soft_priority_score: solution.soft_score,
                total_soft_priority_possible,
                iterations,
            })
        } else {
            let achieved_ratio = if total_required_slots == 0 {
                0.0
            } else {
                max_covered_ever as f64 / total_required_slots as f64
            };

            Ok(SolveResult {
                status: SolveStatus::GoalUnreached,
                schedule: HashMap::new(),
                total_required_slots,
                covered_slots: max_covered_ever,
                coverage_ratio: achieved_ratio,
                target_coverage_ratio,
                target_slots,
                satisfied_rules: Vec::new(),
                dropped_rules: Vec::new(),
                soft_priority_score: 0,
                total_soft_priority_possible,
                iterations,
            })
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn backtrack(
        &self,
        slot_idx: usize,
        covered_slots: usize,
        slots: &[Slot],
        context: &mut ScheduleContext,
        target_slots: usize,
        total_slots: usize,
        total_soft_priority_possible: u64,
        iterations: &mut usize,
        best: &mut Option<BestCandidate>,
        max_covered_ever: &mut usize,
    ) {
        *iterations += 1;
        if *iterations > self.max_iterations {
            return;
        }

        if covered_slots > *max_covered_ever {
            *max_covered_ever = covered_slots;
        }

        let remaining_slots = total_slots - slot_idx;

        // Prune if remaining slots cannot even reach the target
        if covered_slots + remaining_slots < target_slots {
            return;
        }

        // Base case: all slots considered
        if slot_idx == total_slots {
            if covered_slots >= target_slots {
                self.evaluate_leaf(covered_slots, context, total_soft_priority_possible, best);
            }
            return;
        }

        // Early stopping if an optimal solution (100% coverage + all soft rules satisfied) is found
        if let Some(b) = best
            && b.covered_slots == total_slots
            && b.soft_score == total_soft_priority_possible
        {
            return;
        }

        let current_slot = &slots[slot_idx];
        let shift = match context.shifts.get(&current_slot.shift_id) {
            Some(s) => s,
            None => return,
        };

        // Try assigning available candidate workers
        // Order workers by current assigned hours (load balancing heuristic)
        let mut candidate_workers: Vec<&Worker> = self.workers.iter().collect();
        candidate_workers.sort_by(|a, b| {
            let ha = context.worker_hours(&a.id);
            let hb = context.worker_hours(&b.id);
            ha.partial_cmp(&hb).unwrap_or(std::cmp::Ordering::Equal)
        });

        for worker in candidate_workers {
            // Cannot assign same worker twice to same shift
            if context
                .shift_assigned_workers(&shift.id)
                .contains(&worker.id)
            {
                continue;
            }

            // Hard constraints pruning
            let mut can_assign = true;
            for rule in &self.rules {
                if !rule.can_assign(worker, shift, context) {
                    can_assign = false;
                    break;
                }
            }

            if can_assign {
                context.assign(&shift.id, &worker.id);
                self.backtrack(
                    slot_idx + 1,
                    covered_slots + 1,
                    slots,
                    context,
                    target_slots,
                    total_slots,
                    total_soft_priority_possible,
                    iterations,
                    best,
                    max_covered_ever,
                );
                context.unassign(&shift.id, &worker.id);

                if let Some(b) = best
                    && b.covered_slots == total_slots
                    && b.soft_score == total_soft_priority_possible
                {
                    return;
                }
            }
        }

        // Also branch on leaving slot unassigned (if target coverage allows skipping it)
        if (covered_slots + remaining_slots.saturating_sub(1)) >= target_slots {
            self.backtrack(
                slot_idx + 1,
                covered_slots,
                slots,
                context,
                target_slots,
                total_slots,
                total_soft_priority_possible,
                iterations,
                best,
                max_covered_ever,
            );
        }
    }

    fn evaluate_leaf(
        &self,
        covered_slots: usize,
        context: &ScheduleContext,
        _total_soft_priority_possible: u64,
        best: &mut Option<BestCandidate>,
    ) {
        let mut has_hard_violation = false;
        let mut violations = Vec::new();
        let mut satisfied = Vec::new();
        let mut soft_score = 0u64;

        for rule in &self.rules {
            let rule_violations = rule.evaluate(context);
            if rule.priority().is_required() {
                if !rule_violations.is_empty() {
                    has_hard_violation = true;
                    break;
                }
                satisfied.push(RuleSatisfaction {
                    rule_id: rule.id().to_string(),
                    rule_name: rule.name().to_string(),
                    priority: rule.priority(),
                    satisfied: true,
                });
            } else {
                if rule_violations.is_empty() {
                    soft_score += rule.priority().priority_value() as u64;
                    satisfied.push(RuleSatisfaction {
                        rule_id: rule.id().to_string(),
                        rule_name: rule.name().to_string(),
                        priority: rule.priority(),
                        satisfied: true,
                    });
                } else {
                    satisfied.push(RuleSatisfaction {
                        rule_id: rule.id().to_string(),
                        rule_name: rule.name().to_string(),
                        priority: rule.priority(),
                        satisfied: false,
                    });
                    violations.extend(rule_violations);
                }
            }
        }

        if has_hard_violation {
            return;
        }

        let is_better = match best {
            None => true,
            Some(prev) => {
                if soft_score > prev.soft_score {
                    true
                } else if soft_score == prev.soft_score {
                    covered_slots > prev.covered_slots
                } else {
                    false
                }
            }
        };

        if is_better {
            *best = Some(BestCandidate {
                soft_score,
                covered_slots,
                assignments: context.assignments.clone(),
                violations,
                satisfied_rules: satisfied,
            });
        }
    }
}

impl Default for Planner {
    fn default() -> Self {
        Self::new()
    }
}
