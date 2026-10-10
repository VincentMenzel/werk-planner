use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoverageGoal {
    Ratio(f64),
    Percentage(f64),
    MinSlots(usize),
}

impl CoverageGoal {
    pub fn ratio(ratio: f64) -> Self {
        Self::Ratio(ratio.clamp(0.0, 1.0))
    }

    pub fn percentage(pct: f64) -> Self {
        Self::Percentage(pct.clamp(0.0, 100.0))
    }

    pub fn fraction(numerator: usize, denominator: usize) -> Self {
        if denominator == 0 {
            Self::Ratio(1.0)
        } else {
            Self::Ratio((numerator as f64 / denominator as f64).clamp(0.0, 1.0))
        }
    }

    pub fn min_slots(slots: usize) -> Self {
        Self::MinSlots(slots)
    }

    pub fn required_slots(&self, total_slots: usize) -> usize {
        if total_slots == 0 {
            return 0;
        }
        match self {
            Self::Ratio(r) => ((total_slots as f64) * r).ceil() as usize,
            Self::Percentage(p) => ((total_slots as f64) * (p / 100.0)).ceil() as usize,
            Self::MinSlots(s) => (*s).min(total_slots),
        }
    }

    pub fn is_satisfied(&self, covered_slots: usize, total_slots: usize) -> bool {
        covered_slots >= self.required_slots(total_slots)
    }
}

impl Default for CoverageGoal {
    fn default() -> Self {
        Self::Ratio(1.0)
    }
}
