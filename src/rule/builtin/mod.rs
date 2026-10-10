pub mod consecutive_days;
pub mod consecutive_weekdays;
pub mod day_off;
pub mod hours;
pub mod overlap;
pub mod shift_tag;

pub use consecutive_days::MaxConsecutiveDaysRule;
pub use consecutive_weekdays::NoConsecutiveWeekdayRule;
pub use day_off::DayOffRule;
pub use hours::MinMaxHoursRule;
pub use overlap::NoOverlappingShiftsRule;
pub use shift_tag::ShiftTagForbiddenRule;
