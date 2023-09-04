use crate::{
  school_schedule::{class_calendar::ClassCalendar, Availability, SimulationConstraints},
  timeslot,
};

pub(crate) fn same_timeslot_classes_count(
  state: &ClassCalendar,
  _constraints: &SimulationConstraints,
) -> f32 {
  let mut same_timeslot_classes_count: f32 = 0.0;
  for classes in state.get_matrix().iter() {
    let same_timeslot: bool = classes.iter().filter(|x| **x > 1).nth(1).is_some();
    if same_timeslot {
      same_timeslot_classes_count += 1.0;
    }
  }
  same_timeslot_classes_count
}

pub(crate) fn count_not_available(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> f32 {
  let mut not_available_count: f32 = 0.0;
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
) -> f32 {
  let mut available_if_needed_count: f32 = 0.0;
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
