use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use std::collections::HashSet;
use werk_planner::{
    CoverageGoal, DayOffRule, MaxConsecutiveDaysRule, MinMaxHoursRule, NoConsecutiveWeekdayRule,
    NoOverlappingShiftsRule, Planner, Rule, RulePriority, RuleScope, RuleViolation,
    ScheduleContext, Shift, ShiftId, ShiftTagForbiddenRule, SolveStatus, Worker, WorkerId,
};

fn time(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn test_basic_schedule_assignment() {
    let worker1 = Worker::new("w1", "Alice").with_hours(0.0, 40.0);
    let shift1 = Shift::new("s1", date(2026, 10, 12), time(8, 0), time(17, 0));

    let planner = Planner::new()
        .add_worker(worker1)
        .add_shift(shift1)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::Success);
    assert_eq!(result.covered_slots, 1);
    assert_eq!(result.coverage_ratio, 1.0);
    assert_eq!(
        result.assigned_workers(&ShiftId::from("s1")),
        &[WorkerId::from("w1")]
    );
}

#[test]
fn test_worker_hours_limit() {
    // 8-hour shift. Worker max is 7 hours.
    let worker = Worker::new("w1", "Alice").with_hours(0.0, 7.0);
    let shift = Shift::new("s1", date(2026, 10, 12), time(9, 0), time(17, 0)); // 8 hours

    let hours_rule = MinMaxHoursRule::new(
        "hours_limit",
        "Max Hours",
        RulePriority::Required,
        RuleScope::Global,
    );

    let planner = Planner::new()
        .add_worker(worker)
        .add_shift(shift)
        .add_rule(hours_rule)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::GoalUnreached);
    assert_eq!(result.covered_slots, 0);
}

#[test]
fn test_worker_rule_no_saturday() {
    // Saturday is 2026-10-17
    let sat = date(2026, 10, 17);
    assert_eq!(sat.weekday(), Weekday::Sat);

    let alice = Worker::new("w1", "Alice");
    let bob = Worker::new("w2", "Bob");

    let shift_sat = Shift::new("s_sat", sat, time(8, 0), time(17, 0));

    // Alice has Saturday off (attached to Alice)
    let alice_no_sat = DayOffRule::for_weekday(
        "alice_no_sat",
        "Alice No Saturday",
        Weekday::Sat,
        RulePriority::Required,
        RuleScope::Worker(WorkerId::from("w1")),
    );

    let planner = Planner::new()
        .add_worker(alice)
        .add_worker(bob)
        .add_shift(shift_sat)
        .add_rule(alice_no_sat)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::Success);
    assert_eq!(
        result.assigned_workers(&ShiftId::from("s_sat")),
        &[WorkerId::from("w2")]
    );
}

#[test]
fn test_global_rule_sunday_closed() {
    // Sunday is 2026-10-18
    let sun = date(2026, 10, 18);
    assert_eq!(sun.weekday(), Weekday::Sun);

    let worker = Worker::new("w1", "Alice");
    let shift_sun = Shift::new("s_sun", sun, time(8, 0), time(17, 0));

    let sunday_closed = DayOffRule::for_weekday(
        "sunday_closed",
        "Sunday Closed",
        Weekday::Sun,
        RulePriority::Required,
        RuleScope::Global,
    );

    let planner = Planner::new()
        .add_worker(worker)
        .add_shift(shift_sun)
        .add_rule(sunday_closed)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::GoalUnreached);
    assert_eq!(result.covered_slots, 0);
}

#[test]
fn test_not_two_days_in_a_row() {
    let day1 = date(2026, 10, 12); // Mon
    let day2 = date(2026, 10, 13); // Tue

    let alice = Worker::new("w1", "Alice");
    let s1 = Shift::new("s1", day1, time(8, 0), time(17, 0));
    let s2 = Shift::new("s2", day2, time(8, 0), time(17, 0));

    let no_consecutive = MaxConsecutiveDaysRule::new(
        "max_1_day",
        "Max 1 Day In A Row",
        1,
        RulePriority::Required,
        RuleScope::Global,
    );

    // Only Alice available for 2 days in a row: cannot staff both days
    let planner = Planner::new()
        .add_worker(alice)
        .add_shift(s1)
        .add_shift(s2)
        .add_rule(no_consecutive)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::GoalUnreached);

    // With 50% coverage goal (1 of 2 shifts), it succeeds
    let planner_50 = Planner::new()
        .add_worker(Worker::new("w1", "Alice"))
        .add_shift(Shift::new("s1", day1, time(8, 0), time(17, 0)))
        .add_shift(Shift::new("s2", day2, time(8, 0), time(17, 0)))
        .add_rule(MaxConsecutiveDaysRule::new(
            "max_1_day",
            "Max 1 Day In A Row",
            1,
            RulePriority::Required,
            RuleScope::Global,
        ))
        .with_goal(CoverageGoal::fraction(1, 2));

    let result_50 = planner_50.solve().unwrap();
    assert_eq!(result_50.status, SolveStatus::Success);
    assert_eq!(result_50.covered_slots, 1);
}

#[test]
fn test_no_two_saturdays_in_a_row() {
    let sat1 = date(2026, 10, 17);
    let sat2 = date(2026, 10, 24);
    assert_eq!(sat1.weekday(), Weekday::Sat);
    assert_eq!(sat2.weekday(), Weekday::Sat);

    let alice = Worker::new("w1", "Alice");
    let s1 = Shift::new("s1", sat1, time(8, 0), time(17, 0));
    let s2 = Shift::new("s2", sat2, time(8, 0), time(17, 0));

    let no_two_sats = NoConsecutiveWeekdayRule::new(
        "no_two_sats",
        "No Two Saturdays In A Row",
        Weekday::Sat,
        RulePriority::Required,
        RuleScope::Global,
    );

    let planner = Planner::new()
        .add_worker(alice)
        .add_shift(s1)
        .add_shift(s2)
        .add_rule(no_two_sats)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::GoalUnreached);

    // With Bob added, both Saturdays can be covered without anyone working two in a row
    let bob = Worker::new("w2", "Bob");
    let planner_with_bob = Planner::new()
        .add_worker(Worker::new("w1", "Alice"))
        .add_worker(bob)
        .add_shift(Shift::new("s1", sat1, time(8, 0), time(17, 0)))
        .add_shift(Shift::new("s2", sat2, time(8, 0), time(17, 0)))
        .add_rule(NoConsecutiveWeekdayRule::new(
            "no_two_sats",
            "No Two Saturdays In A Row",
            Weekday::Sat,
            RulePriority::Required,
            RuleScope::Global,
        ))
        .with_goal(CoverageGoal::percentage(100.0));

    let result_bob = planner_with_bob.solve().unwrap();
    assert_eq!(result_bob.status, SolveStatus::Success);
    assert_eq!(result_bob.covered_slots, 2);
    let worker_sat1 = &result_bob.assigned_workers(&ShiftId::from("s1"))[0];
    let worker_sat2 = &result_bob.assigned_workers(&ShiftId::from("s2"))[0];
    assert_ne!(worker_sat1, worker_sat2);
}

#[test]
fn test_no_afternoon_tag() {
    let day = date(2026, 10, 12);
    let morning = Shift::new("morning", day, time(8, 0), time(12, 0)).with_tag("morning");
    let afternoon = Shift::new("afternoon", day, time(12, 0), time(17, 0)).with_tag("afternoon");

    let alice = Worker::new("w1", "Alice"); // dislikes afternoons
    let bob = Worker::new("w2", "Bob");

    let no_afternoon_alice = ShiftTagForbiddenRule::new(
        "alice_no_pm",
        "Alice No Afternoons",
        "afternoon",
        RulePriority::Required,
        RuleScope::Worker(WorkerId::from("w1")),
    );

    let planner = Planner::new()
        .add_worker(alice)
        .add_worker(bob)
        .add_shift(morning)
        .add_shift(afternoon)
        .add_rule(no_afternoon_alice)
        .add_rule(NoOverlappingShiftsRule::default())
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::Success);
    assert_eq!(
        result.assigned_workers(&ShiftId::from("afternoon")),
        &[WorkerId::from("w2")]
    );
}

#[test]
fn test_soft_rule_relaxation_lowest_priority_dropped() {
    // 7 days of 8-17 shifts
    // Goal: 6/7 days covered
    // Worker Alice has:
    //   - Soft rule 1: No Saturday (priority 10)
    //   - Soft rule 2: No Sunday (priority 50)
    // To cover 6 out of 7 days, Alice must work either Saturday or Sunday.
    // The planner should drop the lowest priority rule (No Saturday, prio 10)
    // and keep the higher priority rule (No Sunday, prio 50).
    let shifts: Vec<Shift> = (12..=18)
        .map(|d| {
            Shift::new(
                format!("s_{}", d),
                date(2026, 10, d),
                time(8, 0),
                time(17, 0),
            )
        })
        .collect();

    let alice = Worker::new("w1", "Alice").with_hours(0.0, 60.0);

    let no_sat_low_prio = DayOffRule::for_weekday(
        "no_sat",
        "No Saturday",
        Weekday::Sat,
        RulePriority::Soft(10),
        RuleScope::Worker(WorkerId::from("w1")),
    );

    let no_sun_high_prio = DayOffRule::for_weekday(
        "no_sun",
        "No Sunday",
        Weekday::Sun,
        RulePriority::Soft(50),
        RuleScope::Worker(WorkerId::from("w1")),
    );

    let planner = Planner::new()
        .add_worker(alice)
        .with_shifts(shifts)
        .add_rule(no_sat_low_prio)
        .add_rule(no_sun_high_prio)
        .with_goal(CoverageGoal::fraction(6, 7));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::Success);
    assert!(result.covered_slots >= 6);

    // No Saturday (prio 10) was dropped
    assert!(result.was_rule_dropped("no_sat"));
    // No Sunday (prio 50) remained satisfied
    assert!(result.is_rule_satisfied("no_sun"));
    // Sunday shift (day 18) must NOT be assigned to Alice
    assert!(result.assigned_workers(&ShiftId::from("s_18")).is_empty());
}

#[test]
fn test_multi_worker_shifts() {
    let day = date(2026, 10, 12);
    let busy_shift = Shift::new("s_double", day, time(8, 0), time(17, 0)).with_required_workers(2);

    let alice = Worker::new("w1", "Alice");
    let bob = Worker::new("w2", "Bob");

    let planner = Planner::new()
        .add_worker(alice)
        .add_worker(bob)
        .add_shift(busy_shift)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::Success);
    assert_eq!(result.covered_slots, 2);

    let assigned = result.assigned_workers(&ShiftId::from("s_double"));
    assert_eq!(assigned.len(), 2);
    let assigned_set: HashSet<WorkerId> = assigned.iter().cloned().collect();
    assert!(assigned_set.contains(&WorkerId::from("w1")));
    assert!(assigned_set.contains(&WorkerId::from("w2")));
}

#[test]
fn test_custom_rule_implementation() {
    // Custom rule: Worker cannot work on day 15
    #[derive(Debug)]
    struct CustomDayFifteenRule;

    impl Rule for CustomDayFifteenRule {
        fn id(&self) -> &str {
            "no_15th"
        }
        fn name(&self) -> &str {
            "No 15th of Month"
        }
        fn priority(&self) -> RulePriority {
            RulePriority::Required
        }
        fn scope(&self) -> &RuleScope {
            &RuleScope::Global
        }
        fn check_assignment_hard(
            &self,
            _worker: &Worker,
            shift: &Shift,
            _context: &ScheduleContext,
        ) -> bool {
            shift.date.day() != 15
        }
        fn evaluate(&self, context: &ScheduleContext) -> Vec<RuleViolation> {
            let mut v = Vec::new();
            for shift in context.shifts.values() {
                if shift.date.day() == 15 && !context.shift_assigned_workers(&shift.id).is_empty() {
                    v.push(RuleViolation {
                        rule_id: self.id().to_string(),
                        rule_name: self.name().to_string(),
                        priority: self.priority(),
                        worker_id: None,
                        shift_id: Some(shift.id.clone()),
                        message: "Shift on 15th worked".to_string(),
                    });
                }
            }
            v
        }
    }

    let alice = Worker::new("w1", "Alice");
    let s_15 = Shift::new("s15", date(2026, 10, 15), time(8, 0), time(17, 0));

    let planner = Planner::new()
        .add_worker(alice)
        .add_shift(s_15)
        .add_rule(CustomDayFifteenRule)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve().unwrap();
    assert_eq!(result.status, SolveStatus::GoalUnreached);
}

#[test]
fn test_serde_serialization() {
    let worker = Worker::new("w1", "Alice").with_hours(10.0, 35.0);
    let serialized_worker = serde_json::to_string(&worker).unwrap();
    let deserialized_worker: Worker = serde_json::from_str(&serialized_worker).unwrap();
    assert_eq!(worker, deserialized_worker);

    let shift = Shift::new("s1", date(2026, 10, 12), time(8, 0), time(16, 0))
        .with_required_workers(2)
        .with_tag("morning");
    let serialized_shift = serde_json::to_string(&shift).unwrap();
    let deserialized_shift: Shift = serde_json::from_str(&serialized_shift).unwrap();
    assert_eq!(shift, deserialized_shift);
}
