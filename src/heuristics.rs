use crate::{
  school_schedule::{class_calendar::ClassCalendar, Availability, SimulationConstraints},
  timeslot,
};

pub(crate) fn same_timeslot_classes_count(
  state: &ClassCalendar,
  _constraints: &SimulationConstraints,
) -> f64 {
  let mut same_timeslot_classes_count: u64 = 0;
  for classes in state.get_matrix().iter() {
    let x: u64 = classes.iter().map(|a| *a as u64).sum();
    if x >= 2 {
      same_timeslot_classes_count += x;
    }
  }
  same_timeslot_classes_count as f64
}

pub(crate) fn count_not_available(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> f64 {
  let mut not_available_count: f64 = 0.0;
  for day in timeslot::DAY_RANGE {
    for timeslot in timeslot::TIMESLOT_RANGE {
      let classes = state.get_timeslot(day, timeslot);
      for (class_id, _count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = *constraints.get_classes()[class_id].get_professor_id();
        let professor = &constraints.get_professors()[professor_id];
        let availability = professor.availability.get(day, timeslot).unwrap();
        if *availability == Availability::NotAvailable {
          not_available_count += 1.0;
        }
      }
    }
  }

  not_available_count
}

pub(crate) fn count_available_if_needed(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> f64 {
  let mut available_if_needed_count: f64 = 0.0;
  for day in timeslot::DAY_RANGE {
    for timeslot in timeslot::TIMESLOT_RANGE {
      let classes = state.get_timeslot(day, timeslot);
      for (class_id, _count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = *constraints.get_classes()[class_id].get_professor_id();
        let professor = &constraints.get_professors()[professor_id];
        let availability = professor.availability.get(day, timeslot).unwrap();
        if *availability == Availability::AvailableIfNeeded {
          available_if_needed_count += 1.0;
        }
      }
    }
  }
  available_if_needed_count
}
