use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum PlannerError {
    #[error("Shift with id '{0}' not found")]
    ShiftNotFound(String),

    #[error("Worker with id '{0}' not found")]
    WorkerNotFound(String),

    #[error("Invalid shift duration: start time {0} must be before end time {1}")]
    InvalidShiftDuration(String, String),

    #[error("Duplicate worker id '{0}'")]
    DuplicateWorker(String),

    #[error("Duplicate shift id '{0}'")]
    DuplicateShift(String),

    #[error(
        "Coverage goal unsatisfiable: required {target_ratio:.2} ({target_slots} slots), but maximum achievable was {achieved_ratio:.2} ({achieved_slots} slots)"
    )]
    GoalUnsatisfiable {
        target_ratio: f64,
        target_slots: usize,
        achieved_ratio: f64,
        achieved_slots: usize,
    },

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}
