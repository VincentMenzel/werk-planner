# werk-planner

A rules-engine scheduling and work plan optimization library in Rust.

`werk-planner` generates shift schedules across workers and shifts based on hard and soft constraints. When staffing targets cannot be met under full constraints, the engine automatically relaxes the lowest-priority soft rules to achieve the coverage goal.

---

## Features

- **Staffing Model**:
  - Workers with minimum and maximum hours per period.
  - Shifts with customizable start/end times, tags (e.g., `morning`, `afternoon`), and required worker counts (`required_workers`).
- **Rules Engine**:
  - Extensible `Rule` trait for custom scheduling policies.
  - Scoped to global constraints or attached to individual workers.
  - **Hard Constraints** (`RulePriority::Required`): Enforced strictly; branches violating hard rules are pruned.
  - **Soft Constraints** (`RulePriority::Soft(u32)`): Scored by priority; relaxed automatically when required to fulfill the coverage target.
- **Built-in Rules**:
  - `DayOffRule`: Forbid working on specific weekdays or calendar dates (e.g., Sunday closed, individual days off).
  - `MaxConsecutiveDaysRule`: Limit consecutive days worked (e.g., no 2 days in a row).
  - `NoConsecutiveWeekdayRule`: Prevent consecutive occurrences of a specific weekday (e.g., no 2 Saturdays in a row).
  - `ShiftTagForbiddenRule`: Filter shifts by tag (e.g., no afternoon shifts).
  - `MinMaxHoursRule`: Enforce minimum and maximum hours per worker.
  - `NoOverlappingShiftsRule`: Prevent scheduling workers for overlapping shifts on the same day.
- **Coverage Goals**:
  - Defined as percentages (`CoverageGoal::percentage(85.0)`), ratios (`CoverageGoal::fraction(6, 7)`), or minimum slot counts (`CoverageGoal::min_slots(10)`).
- **Diagnostics**:
  - Complete report on satisfied rules and dropped rules with reasons.
  - Serde support for all models and solve results.

---

## Quickstart

Add `werk-planner` to `Cargo.toml`:

```toml
[dependencies]
werk-planner = { path = "." }
chrono = "0.4"
```

### Example

```rust
use chrono::{NaiveDate, NaiveTime, Weekday};
use werk_planner::{
    CoverageGoal, DayOffRule, Planner, RulePriority, RuleScope, Shift,
    ShiftTagForbiddenRule, SolveStatus, Worker, WorkerId,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let date = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
    let start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

    let alice = Worker::new("w1", "Alice").with_hours(0.0, 40.0);
    let bob = Worker::new("w2", "Bob").with_hours(0.0, 40.0);

    let shift = Shift::new("s1", date, start, end).with_tag("morning");

    // Worker rule: Alice cannot work Saturdays
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
        .add_shift(shift)
        .add_rule(alice_no_sat)
        .with_goal(CoverageGoal::percentage(100.0));

    let result = planner.solve()?;
    assert_eq!(result.status, SolveStatus::Success);
    println!("Covered slots: {}/{}", result.covered_slots, result.total_required_slots);

    Ok(())
}
```

---

## Development

Tool versioning is managed via [mise](https://mise.jdx.dev/).

```bash
# Run tests
mise run test

# Run clippy
mise run lint

# Check formatting
mise run fmt-check

# Format code
mise run fmt
```
