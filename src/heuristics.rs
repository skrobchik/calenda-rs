use crate::{
  class_filter::ClassFilter,
  school_schedule::{
    class_calendar::{ClassCalendar, ClassId, NUM_CLASS_IDS},
    Availability, ClassroomType, SimulationConstraints,
  },
  week_calendar,
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
      if let Some(class) = simulation_constraints.get_class(class_id.try_into().unwrap()) {
        let professor_id = class.get_professor_id();
        professor_class_counter[*professor_id] += *count as u32;
      }
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
      if let Some(class) = simulation_constraints.get_class(class_id.try_into().unwrap()) {
        let semester = class.get_semester();
        let semester: u32 = semester.into();
        semester_class_counter[semester as usize] += *count as u32;
      }
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
      .filter(|(class_id, _count)| {
        class_filter.filter((*class_id).try_into().unwrap(), simulation_constraints)
      })
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
  for day in week_calendar::Day::all() {
    for timeslot in week_calendar::Timeslot::all() {
      let classes = state.get_timeslot(day, timeslot);
      for (class_id, _count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = *constraints
          .get_class(class_id.try_into().unwrap())
          .unwrap()
          .get_professor_id();
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
  for day in week_calendar::Day::all() {
    for timeslot in week_calendar::Timeslot::all() {
      let classes = state.get_timeslot(day, timeslot);
      for (class_id, _count) in classes.iter().enumerate().filter(|(_, c)| **c > 0) {
        let professor_id = *constraints
          .get_class(class_id.try_into().unwrap())
          .unwrap()
          .get_professor_id();
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
  for day_idx in week_calendar::Day::all() {
    let max_class_id = week_calendar::Timeslot::all()
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
    for timeslot_idx in week_calendar::Timeslot::all() {
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
  let mut class_days: Vec<u8> = vec![0; NUM_CLASS_IDS]; // Counts the number of days in which the i-th class is present
  for day in week_calendar::Day::all() {
    for class_id in ClassId::all() {
      let mut class_found: bool = false;
      for timeslot in week_calendar::Timeslot::all() {
        if state.get_count(day, timeslot, class_id) > 0 {
          class_found = true;
          break;
        }
      }
      if class_found {
        class_days[usize::from(class_id)] += 1;
      }
    }
  }

  let mut inconsistent_count = 0;
  for class_id in ClassId::all() {
    if class_days[usize::from(class_id)] < 2 {
      continue;
    }
    for timeslot in week_calendar::Timeslot::all() {
      let mut count = 0;
      for day in week_calendar::Day::all() {
        if state.get_count(day, timeslot, class_id) > 0 {
          count += 1;
        }
      }
      if count == 1 {
        inconsistent_count += 1;
      }
    }
  }

  inconsistent_count as f64
}

pub(crate) fn count_labs_on_different_days(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> f64 {
  let mut different_days_labs_count = 0;
  for (class_id, class) in constraints.iter_classes_with_id() {
    if match class.get_classroom_type() {
      ClassroomType::AulaSimple => true,
      ClassroomType::AulaDoble => true,
      ClassroomType::LabQuimica => false,
      ClassroomType::LabFisica => false,
      ClassroomType::AulaComputo => false,
    } {
      continue;
    }
    let mut count: i32 = 0;
    for day in week_calendar::Day::all() {
      if week_calendar::Timeslot::all()
        .map(|timeslot| state.get_count(day, timeslot, class_id))
        .any(|c| c >= 1)
      {
        count += 1;
      }
    }
    if count >= 2 {
      different_days_labs_count += count - 1;
    }
  }
  different_days_labs_count as f64
}

#[cfg(test)]
mod test {
  use crate::{
    school_schedule::SchoolSchedule,
    week_calendar::{TIMESLOT_09_00, TIMESLOT_11_00},
  };

  use self::week_calendar::TIMESLOT_08_00;

  use super::*;

  #[test]
  fn count_outside_session_length_test() {
    let mut state = ClassCalendar::default();
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      week_calendar::TIMESLOT_15_00.try_into().unwrap(),
      0.try_into().unwrap(),
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 1.0);
    state.add_one_class(
      0.try_into().unwrap(),
      week_calendar::TIMESLOT_16_00.try_into().unwrap(),
      0.try_into().unwrap(),
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      week_calendar::TIMESLOT_17_00.try_into().unwrap(),
      0.try_into().unwrap(),
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      week_calendar::TIMESLOT_18_00.try_into().unwrap(),
      0.try_into().unwrap(),
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      week_calendar::TIMESLOT_19_00.try_into().unwrap(),
      0.try_into().unwrap(),
    );
    assert_eq!(count_outside_session_length(&state, 2, 4), 1.0);
  }

  #[test]
  fn count_inconsistent_class_timeslots_test() {
    let mut state = ClassCalendar::default();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
    state.add_one_class(
      0.try_into().unwrap(),
      week_calendar::TIMESLOT_18_00.try_into().unwrap(),
      7.try_into().unwrap(),
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
    state.add_one_class(
      4.try_into().unwrap(),
      week_calendar::TIMESLOT_18_00.try_into().unwrap(),
      6.try_into().unwrap(),
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
    state.add_one_class(
      4.try_into().unwrap(),
      week_calendar::TIMESLOT_18_00.try_into().unwrap(),
      7.try_into().unwrap(),
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
    state.add_one_class(
      3.try_into().unwrap(),
      week_calendar::TIMESLOT_19_00.try_into().unwrap(),
      6.try_into().unwrap(),
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 2.0);
    state.add_one_class(
      3.try_into().unwrap(),
      week_calendar::TIMESLOT_18_00.try_into().unwrap(),
      6.try_into().unwrap(),
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 1.0);
    state.add_one_class(
      0.try_into().unwrap(),
      week_calendar::TIMESLOT_19_00.try_into().unwrap(),
      6.try_into().unwrap(),
    );
    assert_eq!(count_inconsistent_class_timeslots(&state), 0.0);
  }

  #[test]
  fn count_labs_on_different_days_test() {
    let mut schedule = SchoolSchedule::default();
    let class_id_0 = schedule.add_new_class();
    let mut class_0 = schedule.get_class_entry_mut(class_id_0).unwrap();
    class_0.set_hours(3);
    class_0.set_classroom_type(ClassroomType::AulaSimple);
    let class_id_1 = schedule.add_new_class();
    let mut class_1 = schedule.get_class_entry_mut(class_id_1).unwrap();
    class_1.set_classroom_type(ClassroomType::LabFisica);
    class_1.set_hours(3);
    let mut state = ClassCalendar::default();
    state.add_one_class(
      0.try_into().unwrap(),
      TIMESLOT_08_00.try_into().unwrap(),
      class_id_0,
    );
    state.add_one_class(
      1.try_into().unwrap(),
      TIMESLOT_08_00.try_into().unwrap(),
      class_id_0,
    );
    state.add_one_class(
      2.try_into().unwrap(),
      TIMESLOT_08_00.try_into().unwrap(),
      class_id_0,
    );
    state.add_one_class(
      0.try_into().unwrap(),
      TIMESLOT_08_00.try_into().unwrap(),
      class_id_1,
    );
    state.add_one_class(
      0.try_into().unwrap(),
      TIMESLOT_09_00.try_into().unwrap(),
      class_id_1,
    );
    state.add_one_class(
      0.try_into().unwrap(),
      TIMESLOT_11_00.try_into().unwrap(),
      class_id_1,
    );
    schedule.replace_class_calendar(state.clone()).unwrap();
    assert_eq!(
      count_labs_on_different_days(
        schedule.get_class_calendar(),
        schedule.get_simulation_constraints()
      ),
      0.0
    );
    state.move_one_class(
      0.try_into().unwrap(),
      TIMESLOT_09_00.try_into().unwrap(),
      1.try_into().unwrap(),
      TIMESLOT_08_00.try_into().unwrap(),
      class_id_1,
    );
    schedule.replace_class_calendar(state).unwrap();
    assert_eq!(
      count_labs_on_different_days(
        schedule.get_class_calendar(),
        schedule.get_simulation_constraints()
      ),
      1.0
    );
  }
}
