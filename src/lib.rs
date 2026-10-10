pub mod error;
pub mod model;
pub mod rule;
pub mod solver;

pub use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
pub use error::PlannerError;
pub use model::{CoverageGoal, Shift, ShiftId, Worker, WorkerId};
pub use rule::{
    Rule, RulePriority, RuleScope, RuleViolation, ScheduleContext,
    builtin::{
        DayOffRule, MaxConsecutiveDaysRule, MinMaxHoursRule, NoConsecutiveWeekdayRule,
        NoOverlappingShiftsRule, ShiftTagForbiddenRule,
    },
};
pub use solver::{Planner, SolveResult, SolveStatus};
