use crate::model::{Shift, ShiftId, Worker, WorkerId};
use chrono::NaiveDate;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone)]
pub struct ScheduleContext<'a> {
    pub workers: &'a HashMap<WorkerId, Worker>,
    pub shifts: &'a HashMap<ShiftId, Shift>,
    pub shift_order: &'a [ShiftId],
    pub assignments: HashMap<ShiftId, Vec<WorkerId>>,
    pub worker_assignments: HashMap<WorkerId, Vec<ShiftId>>,
    pub worker_dates: HashMap<WorkerId, BTreeSet<NaiveDate>>,
    pub worker_hours: HashMap<WorkerId, f64>,
}

impl<'a> ScheduleContext<'a> {
    pub fn new(
        workers: &'a HashMap<WorkerId, Worker>,
        shifts: &'a HashMap<ShiftId, Shift>,
        shift_order: &'a [ShiftId],
    ) -> Self {
        let mut worker_assignments = HashMap::new();
        let mut worker_dates = HashMap::new();
        let mut worker_hours = HashMap::new();

        for worker_id in workers.keys() {
            worker_assignments.insert(worker_id.clone(), Vec::new());
            worker_dates.insert(worker_id.clone(), BTreeSet::new());
            worker_hours.insert(worker_id.clone(), 0.0);
        }

        let mut assignments = HashMap::new();
        for shift_id in shifts.keys() {
            assignments.insert(shift_id.clone(), Vec::new());
        }

        Self {
            workers,
            shifts,
            shift_order,
            assignments,
            worker_assignments,
            worker_dates,
            worker_hours,
        }
    }

    pub fn assign(&mut self, shift_id: &ShiftId, worker_id: &WorkerId) {
        if let Some(assigned) = self.assignments.get_mut(shift_id) {
            assigned.push(worker_id.clone());
        }

        if let Some(shift) = self.shifts.get(shift_id) {
            if let Some(list) = self.worker_assignments.get_mut(worker_id) {
                list.push(shift_id.clone());
            }
            if let Some(dates) = self.worker_dates.get_mut(worker_id) {
                dates.insert(shift.date);
            }
            if let Some(hours) = self.worker_hours.get_mut(worker_id) {
                *hours += shift.duration_hours();
            }
        }
    }

    pub fn unassign(&mut self, shift_id: &ShiftId, worker_id: &WorkerId) {
        if let Some(assigned) = self.assignments.get_mut(shift_id)
            && let Some(pos) = assigned.iter().rposition(|w| w == worker_id)
        {
            assigned.remove(pos);
        }

        if let Some(shift) = self.shifts.get(shift_id) {
            if let Some(list) = self.worker_assignments.get_mut(worker_id)
                && let Some(pos) = list.iter().rposition(|s| s == shift_id)
            {
                list.remove(pos);
            }

            // Re-evaluate if worker still works on that date
            let still_works_on_date = self
                .worker_assignments
                .get(worker_id)
                .map(|shifts| {
                    shifts
                        .iter()
                        .any(|sid| self.shifts.get(sid).is_some_and(|s| s.date == shift.date))
                })
                .unwrap_or(false);

            if !still_works_on_date && let Some(dates) = self.worker_dates.get_mut(worker_id) {
                dates.remove(&shift.date);
            }

            if let Some(hours) = self.worker_hours.get_mut(worker_id) {
                *hours -= shift.duration_hours();
                if *hours < 0.0 {
                    *hours = 0.0;
                }
            }
        }
    }

    pub fn worker_hours(&self, worker_id: &WorkerId) -> f64 {
        self.worker_hours.get(worker_id).copied().unwrap_or(0.0)
    }

    pub fn worker_dates(&self, worker_id: &WorkerId) -> Option<&BTreeSet<NaiveDate>> {
        self.worker_dates.get(worker_id)
    }

    pub fn worker_shifts(&self, worker_id: &WorkerId) -> Vec<&Shift> {
        self.worker_assignments
            .get(worker_id)
            .map(|ids| ids.iter().filter_map(|id| self.shifts.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn shift_assigned_workers(&self, shift_id: &ShiftId) -> &[WorkerId] {
        self.assignments
            .get(shift_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn total_assigned_slots(&self) -> usize {
        self.assignments.values().map(|v| v.len()).sum()
    }

    pub fn total_required_slots(&self) -> usize {
        self.shifts.values().map(|s| s.required_workers).sum()
    }

    pub fn coverage_ratio(&self) -> f64 {
        let total = self.total_required_slots();
        if total == 0 {
            1.0
        } else {
            self.total_assigned_slots() as f64 / total as f64
        }
    }

    pub fn has_overlap(&self, worker_id: &WorkerId, candidate_shift: &Shift) -> bool {
        if let Some(shift_ids) = self.worker_assignments.get(worker_id) {
            for sid in shift_ids {
                if let Some(other) = self.shifts.get(sid)
                    && candidate_shift.overlaps_with(other)
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn dates_by_worker(&self) -> &HashMap<WorkerId, BTreeSet<NaiveDate>> {
        &self.worker_dates
    }

    pub fn assignments_by_shift(&self) -> &HashMap<ShiftId, Vec<WorkerId>> {
        &self.assignments
    }

    pub fn shifts_by_date(&self) -> BTreeMap<NaiveDate, Vec<&Shift>> {
        let mut map: BTreeMap<NaiveDate, Vec<&Shift>> = BTreeMap::new();
        for shift in self.shifts.values() {
            map.entry(shift.date).or_default().push(shift);
        }
        map
    }
}
