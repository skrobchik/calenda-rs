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
  for classes in state.iter_timeslots() {
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
  for classes in state.iter_timeslots() {
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
  simulation_constraints: &SimulationConstraints,
  class_filter: &ClassFilter,
) -> f64 {
  let mut same_timeslot_classes_count: u32 = 0;
  for classes in state.iter_timeslots() {
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
  for day in timeslot::Day::all() {
    for timeslot in timeslot::Timeslot::all() {
      let classes = state.get_timeslot(day, timeslot);
      for (class_id, _count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = *constraints.get_classes()[class_id].get_professor_id();
        let professor = &constraints.get_professors()[professor_id];
        let availability = professor.availability.get(day, timeslot);
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
  for day in timeslot::Day::all() {
    for timeslot in timeslot::Timeslot::all() {
      let classes = state.get_timeslot(day, timeslot);
      for (class_id, _count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = *constraints.get_classes()[class_id].get_professor_id();
        let professor = &constraints.get_professors()[professor_id];
        let availability = professor.availability.get(day, timeslot);
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
  for day_idx in timeslot::Day::all() {
    let max_class_id = timeslot::Timeslot::all()
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
    for timeslot_idx in timeslot::Timeslot::all() {
      let timeslot = state.get_timeslot(day_idx, timeslot_idx);
      for (class_id, class_session_length) in
        session_length.iter_mut().enumerate().take(max_class_id + 1)
      {
        let count = timeslot.get(class_id).copied().unwrap_or_default();
        if count > 0 {
          *class_session_length += 1;
        } else if *class_session_length > 0 {
          if *class_session_length < min_session_length
            || max_session_length < *class_session_length
          {
            outside_session_length_count += 1;
          }
          *class_session_length = 0;
        }
      }
    }
    for &class_session_length in session_length.iter().take(max_class_id + 1) {
      if class_session_length > 0
        && (class_session_length < min_session_length || max_session_length < class_session_length)
      {
        outside_session_length_count += 1;
      }
    }
  }
  outside_session_length_count as f64
}

pub(crate) fn count_inconsistent_class_timeslots(state: &ClassCalendar) -> f64 {
  let max_class_id_plus_one = state
    .iter_timeslots()
    .map(|timeslot| timeslot.len())
    .max()
    .unwrap();
  let mut class_count: Vec<Vec<u16>> =
    vec![vec![0; max_class_id_plus_one]; timeslot::TIMESLOT_COUNT];
  for day_idx in timeslot::Day::all() {
    for timeslot_idx in timeslot::Timeslot::all() {
      class_count[Into::<usize>::into(timeslot_idx)]
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
  use super::*;

  #[test]
  fn count_outside_session_length_test() {
    let mut state = ClassCalendar::default();
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      timeslot::TIMESLOT_15_00.try_into().unwrap(),
      0,
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 1.0);
    state.add_one_class(
      0.try_into().unwrap(),
      timeslot::TIMESLOT_16_00.try_into().unwrap(),
      0,
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      timeslot::TIMESLOT_17_00.try_into().unwrap(),
      0,
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      timeslot::TIMESLOT_18_00.try_into().unwrap(),
      0,
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      timeslot::TIMESLOT_19_00.try_into().unwrap(),
      0,
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 1.0);
  }

  #[test]
  fn count_inconsistent_class_timeslots_test() {
    let mut state = ClassCalendar::default();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      timeslot::TIMESLOT_18_00.try_into().unwrap(),
      7,
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 1.0);
    state.add_one_class(
      4.try_into().unwrap(),
      timeslot::TIMESLOT_18_00.try_into().unwrap(),
      6,
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 2.0);
    state.add_one_class(
      4.try_into().unwrap(),
      timeslot::TIMESLOT_18_00.try_into().unwrap(),
      7,
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 1.0);
    state.add_one_class(
      3.try_into().unwrap(),
      timeslot::TIMESLOT_19_00.try_into().unwrap(),
      6,
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 2.0);
    state.add_one_class(
      3.try_into().unwrap(),
      timeslot::TIMESLOT_18_00.try_into().unwrap(),
      6,
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 1.0);
    state.add_one_class(
      0.try_into().unwrap(),
      timeslot::TIMESLOT_19_00.try_into().unwrap(),
      6,
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
  }
}
