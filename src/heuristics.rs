use crate::{
  school_schedule::{Availability, SimulationConstraints, TimeslotClassHours},
  timeslot,
  week_calendar::WeekCalendar,
};

pub fn same_timeslot_classes_count(
  state: &WeekCalendar<TimeslotClassHours>,
  constraints: &SimulationConstraints,
) -> f32 {
  let mut same_timeslot_classes_count: f32 = 0.0;
  for classes in state.data.iter() {
    let same_timeslot: bool = classes.iter().filter(|x| **x > 1).nth(1).is_some();
    if same_timeslot {
      same_timeslot_classes_count += 1.0;
    }
  }
  same_timeslot_classes_count
}

pub fn count_not_available(
  state: &WeekCalendar<TimeslotClassHours>,
  constraints: &SimulationConstraints,
) -> f32 {
  let mut not_available_count: f32 = 0.0;
  for day in timeslot::DAY_RANGE {
    for timeslot in timeslot::TIMESLOT_RANGE {
      let classes = state.get(day, timeslot).unwrap();
      for (class_id, count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = constraints.classes[class_id].professor;
        let professor = &constraints.professors[professor_id];
        let availability = professor.availability.get(day, timeslot).unwrap();
        if *availability == Availability::NotAvailable {
          not_available_count += 1.0;
        }
      }
    }
  }

  not_available_count
}

pub fn count_available_if_needed(
  state: &WeekCalendar<TimeslotClassHours>,
  constraints: &SimulationConstraints,
) -> f32 {
  let mut available_if_needed_count: f32 = 0.0;
  for day in timeslot::DAY_RANGE {
    for timeslot in timeslot::TIMESLOT_RANGE {
      let classes = state.get(day, timeslot).unwrap();
      for (class_id, count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = constraints.classes[class_id].professor;
        let professor = &constraints.professors[professor_id];
        let availability = professor.availability.get(day, timeslot).unwrap();
        if *availability == Availability::AvailableIfNeeded {
          available_if_needed_count += 1.0;
        }
      }
    }
  }
  available_if_needed_count
}
