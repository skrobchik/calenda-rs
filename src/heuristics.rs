use crate::{
  class_filter::ClassFilter,
  school_schedule::{class_calendar::ClassCalendar, Availability, SimulationConstraints},
  timeslot,
};

pub(crate) fn same_timeslot_classes_count_per_professor(
  state: &ClassCalendar,
  simulation_constraints: &SimulationConstraints,
) -> f64 {
  let mut same_timeslot_classes_count: u32 = 0;
  let num_professors = simulation_constraints.get_professors().len();
  let mut professor_class_counter = vec![0_u32; num_professors];
  for classes in state.get_matrix().iter() {
    professor_class_counter.fill(0);
    for (class_id, count) in classes.iter().enumerate() {
      let professor_id = simulation_constraints
        .get_classes()
        .get(class_id)
        .unwrap()
        .get_professor_id();
      professor_class_counter[*professor_id] += *count as u32;
    }
    same_timeslot_classes_count += professor_class_counter
      .iter()
      .filter(|x| **x >= 2)
      .sum::<u32>();
  }
  same_timeslot_classes_count as f64
}

pub(crate) fn same_timeslot_classes_count_per_semester(
  state: &ClassCalendar,
  simulation_constraints: &SimulationConstraints,
) -> f64 {
  let mut same_timeslot_classes_count: u32 = 0;
  const NUM_SEMESTERS: usize = 8;
  let mut semester_class_counter = [0_u32; NUM_SEMESTERS];
  for classes in state.get_matrix().iter() {
    semester_class_counter.fill(0);
    for (class_id, count) in classes.iter().enumerate() {
      let semester = simulation_constraints
        .get_classes()
        .get(class_id)
        .unwrap()
        .get_semester();
      let semester: u32 = semester.into();
      semester_class_counter[semester as usize] += *count as u32;
    }
    same_timeslot_classes_count += semester_class_counter
      .iter()
      .filter(|x| **x >= 2)
      .sum::<u32>();
  }
  same_timeslot_classes_count as f64
}

pub(crate) fn same_timeslot_classes_count(
  state: &ClassCalendar,
  class_filter: &ClassFilter,
  simulation_constraints: &SimulationConstraints,
) -> f64 {
  let mut same_timeslot_classes_count: u32 = 0;
  for classes in state.get_matrix().iter() {
    let x: u32 = classes
      .iter()
      .enumerate()
      .filter(|(class_id, _count)| class_filter.filter(*class_id, simulation_constraints))
      .map(|(_class_id, count)| *count as u32)
      .sum();
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

pub(crate) fn count_outside_session_length(
  state: &ClassCalendar,
  min_session_length: u8,
  max_session_length: u8,
) -> f64 {
  let mut outside_session_length_count: u64 = 0;
  for day_idx in timeslot::DAY_RANGE {
    let max_class_id = timeslot::TIMESLOT_RANGE
      .map(|timeslot_idx| {
        state
          .get_timeslot(day_idx, timeslot_idx)
          .len()
          .checked_sub(1)
          .unwrap_or_default()
      })
      .max()
      .unwrap();
    let mut session_length: Vec<u8> = vec![0; max_class_id + 1];
    for timeslot_idx in timeslot::TIMESLOT_RANGE {
      let timeslot = state.get_timeslot(day_idx, timeslot_idx);
      for class_id in 0..=max_class_id {
        let count = timeslot.get(class_id).copied().unwrap_or_default();
        if count > 0 {
          session_length[class_id] += 1;
        } else if session_length[class_id] > 0 {
          if session_length[class_id] < min_session_length
            || max_session_length < session_length[class_id]
          {
            outside_session_length_count += 1;
          }
          session_length[class_id] = 0;
        }
      }
    }
    for class_id in 0..=max_class_id {
      if session_length[class_id] > 0
        && (session_length[class_id] < min_session_length
          || max_session_length < session_length[class_id])
      {
        outside_session_length_count += 1;
      }
    }
  }
  outside_session_length_count as f64
}

pub(crate) fn count_inconsistent_class_timeslots(state: &ClassCalendar) -> f64 {
  let max_class_id_plus_one = state
    .get_matrix()
    .iter()
    .map(|timeslot| timeslot.len())
    .max()
    .unwrap();
  let mut class_count: Vec<Vec<u16>> =
    vec![vec![0; max_class_id_plus_one]; timeslot::TIMESLOT_COUNT];
  for day_idx in timeslot::DAY_RANGE {
    for timeslot_idx in timeslot::TIMESLOT_RANGE {
      class_count[timeslot_idx]
        .iter_mut()
        .zip(state.get_timeslot(day_idx, timeslot_idx).iter())
        .for_each(|(x, c)| {
          *x += *c as u16;
        });
    }
  }
  let result = class_count.iter().flatten().filter(|x| **x == 1).count();
  result as f64
}

#[cfg(test)]
mod test {
  use crate::timeslot::{TIMESLOT_19_00, TIMESLOT_19_30};

  use super::*;

  #[test]
  fn count_outside_session_length_test() {
    let mut state = ClassCalendar::new();
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(0, timeslot::TIMESLOT_17_30, 0);
    assert_eq!(count_outside_session_length(&state, 2, 4), 1.0);
    state.add_one_class(0, timeslot::TIMESLOT_18_00, 0);
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(0, timeslot::TIMESLOT_18_30, 0);
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(0, timeslot::TIMESLOT_19_00, 0);
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(0, timeslot::TIMESLOT_19_30, 0);
    assert_eq!(count_outside_session_length(&state, 2, 4), 1.0);
  }

  #[test]
  fn count_inconsistent_class_timeslots_test() {
    let mut state = ClassCalendar::new();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
    state.add_one_class(0, TIMESLOT_19_30, 7);
    assert_eq!(count_inconsistent_class_timeslots(&state), 1.0);
    state.add_one_class(4, TIMESLOT_19_30, 6);
    assert_eq!(count_inconsistent_class_timeslots(&state), 2.0);
    state.add_one_class(4, TIMESLOT_19_30, 7);
    assert_eq!(count_inconsistent_class_timeslots(&state), 1.0);
    state.add_one_class(3, TIMESLOT_19_00, 6);
    assert_eq!(count_inconsistent_class_timeslots(&state), 2.0);
    state.add_one_class(3, TIMESLOT_19_30, 6);
    assert_eq!(count_inconsistent_class_timeslots(&state), 1.0);
    state.add_one_class(0, TIMESLOT_19_00, 6);
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
  }
}
