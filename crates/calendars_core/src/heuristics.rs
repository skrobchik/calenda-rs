use itertools::Itertools;
use slotmap::SecondaryMap;

use crate::{
  school_schedule::{
    class_calendar::{ClassCalendar, ClassKey},
    Availability, ClassroomType, ProfessorKey, SimulationConstraints,
  },
  week_calendar,
};

fn iter_class_calendar(
  class_calendar: &ClassCalendar,
) -> impl Iterator<Item = (ClassKey, week_calendar::Day, week_calendar::Timeslot)> + '_ {
  let classes = class_calendar.iter_class_keys();

  classes.flat_map(move |class_key| {
    week_calendar::Day::all().flat_map(move |day| {
      week_calendar::Timeslot::all().map(move |timeslot| (class_key, day, timeslot))
    })
  })
}

fn iter_week() -> impl Iterator<Item = (week_calendar::Day, week_calendar::Timeslot)> {
  week_calendar::Day::all().flat_map(|d| week_calendar::Timeslot::all().map(move |t| (d, t)))
}

pub(crate) fn same_timeslot_classes_count_per_professor(
  state: &ClassCalendar,
  simulation_constraints: &SimulationConstraints,
) -> u64 {
  let mut same_timeslot_classes_count: u64 = 0;
  let mut professor_class_counter: SecondaryMap<ProfessorKey, u64> = SecondaryMap::from_iter(
    simulation_constraints
      .get_professors()
      .keys()
      .map(|k| (k, 0)),
  );
  for (day, timeslot) in iter_week() {
    professor_class_counter.values_mut().for_each(|x| *x = 0);
    for class_key in state.iter_class_keys() {
      let count = state.get_count(day, timeslot, class_key);
      let class = simulation_constraints.get_class(class_key).unwrap();
      let professor_key = class.get_professor_id();
      professor_class_counter[professor_key] += count as u64;
    }
    same_timeslot_classes_count += professor_class_counter
      .iter()
      .map(|(_k, &v)| v)
      .filter(|&x| x >= 2)
      .sum::<u64>();
  }
  same_timeslot_classes_count
}

pub(crate) fn same_timeslot_classes_count_per_semester(
  state: &ClassCalendar,
  simulation_constraints: &SimulationConstraints,
) -> u64 {
  let mut same_timeslot_classes_count: u64 = 0;
  const NUM_SEMESTERS: usize = 8;
  let mut semester_class_counter = [0_u64; NUM_SEMESTERS + 1];
  for (class_key, day, timeslot) in iter_class_calendar(state) {
    semester_class_counter.fill(0);
    let count = state.get_count(day, timeslot, class_key);
    if let Some(class) = simulation_constraints.get_class(class_key) {
      let semester = class.get_semester();
      let semester: u32 = semester.into();
      semester_class_counter[semester as usize] += count as u64;
    }
    same_timeslot_classes_count += semester_class_counter
      .iter()
      .filter(|x| **x >= 2)
      .sum::<u64>();
  }
  same_timeslot_classes_count
}

pub(crate) fn same_timeslot_classes_count(state: &ClassCalendar) -> u64 {
  let mut same_timeslot_classes_count: u64 = 0;
  for class_key in state.iter_class_keys() {
    for timeslot in week_calendar::Timeslot::all() {
      let x: u64 = week_calendar::Day::all()
        .map(|day| state.get_count(day, timeslot, class_key) as u64)
        .sum();
      if x >= 2 {
        same_timeslot_classes_count += x;
      }
    }
  }
  same_timeslot_classes_count
}

pub(crate) fn count_not_available(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> u64 {
  let mut not_available_count: u64 = 0;

  for (class_key, day, timeslot) in iter_class_calendar(state) {
    let professor_id = constraints.get_class(class_key).unwrap().get_professor_id();
    let professor = &constraints.get_professors()[professor_id];
    let availability = professor.availability.get(day, timeslot);
    if matches!(availability, Availability::NotAvailable) {
      not_available_count += 1;
    }
  }

  not_available_count
}

pub(crate) fn count_available_if_needed(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> u64 {
  let mut available_if_needed_count: u64 = 0;

  for (class_key, day, timeslot) in iter_class_calendar(state) {
    let professor_id = constraints.get_class(class_key).unwrap().get_professor_id();
    let professor = &constraints.get_professors()[professor_id];
    let availability = professor.availability.get(day, timeslot);
    if matches!(availability, Availability::AvailableIfNeeded) {
      available_if_needed_count += 1;
    }
  }

  available_if_needed_count
}

pub(crate) fn count_outside_session_length(
  state: &ClassCalendar,
  min_session_length: u8,
  max_session_length: u8,
) -> u64 {
  let mut outside_session_length_count: u64 = 0;
  for class_key in state.iter_class_keys() {
    for day in week_calendar::Day::all() {
      let mut session_length: u8 = 0;
      for timeslot in week_calendar::Timeslot::all() {
        if state.get_count(day, timeslot, class_key) > 0 {
          session_length += 1;
        } else if session_length > 0 {
          if session_length < min_session_length || max_session_length < session_length {
            outside_session_length_count += 1;
          }
          session_length = 0;
        } else {
          assert_eq!(session_length, 0);
        }
      }
      if session_length >= 1
        && (session_length < min_session_length || max_session_length < session_length)
      {
        outside_session_length_count += 1;
      }
    }
  }
  outside_session_length_count
}

pub(crate) fn count_inconsistent_class_timeslots(state: &ClassCalendar) -> u64 {
  let mut class_days: SecondaryMap<ClassKey, u8> = Default::default(); // Counts the number of days in which the i-th class is present
  for class_key in state.iter_class_keys() {
    class_days.insert(class_key, 0);
  }
  for day in week_calendar::Day::all() {
    for class_key in state.iter_class_keys() {
      let mut class_found: bool = false;
      for timeslot in week_calendar::Timeslot::all() {
        if state.get_count(day, timeslot, class_key) > 0 {
          class_found = true;
          break;
        }
      }
      if class_found {
        class_days[class_key] += 1;
      }
    }
  }

  let mut inconsistent_count = 0;
  for class_key in state.iter_class_keys() {
    if class_days[class_key] < 2 {
      continue;
    }
    for timeslot in week_calendar::Timeslot::all() {
      let mut count = 0;
      for day in week_calendar::Day::all() {
        if state.get_count(day, timeslot, class_key) > 0 {
          count += 1;
        }
      }
      if count == 1 {
        inconsistent_count += 1;
      }
    }
  }

  inconsistent_count
}

pub(crate) fn count_labs_on_different_days(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> u64 {
  let mut different_days_labs_count = 0;
  let class_keys = state.iter_class_keys();
  for (class_key, class) in class_keys.map(|k| (k, constraints.get_class(k).unwrap())) {
    if class
      .get_allowed_classroom_types()
      .intersection_c(ClassroomType::LabFisica | ClassroomType::LabQuimica)
      .is_empty()
    {
      continue;
    }
    let mut count: u64 = 0;
    for day in week_calendar::Day::all() {
      if week_calendar::Timeslot::all()
        .map(|timeslot| state.get_count(day, timeslot, class_key))
        .any(|c| c >= 1)
      {
        count += 1;
      }
    }
    if count >= 2 {
      different_days_labs_count += count - 1;
    }
  }
  different_days_labs_count
}

pub(crate) fn count_incontinuous_classes(state: &ClassCalendar) -> u64 {
  let mut count = 0;
  for class_key in state.iter_class_keys() {
    for day in week_calendar::Day::all() {
      let times = week_calendar::Timeslot::all()
        .enumerate()
        .map(|(i, t)| (i, state.get_count(day, t, class_key)))
        .filter(|(_i, c)| *c >= 1)
        .map(|(i, _c)| i);
      if times.tuple_windows().any(|(i1, i2)| {
        assert!(i1 < i2);
        i1 + 1 < i2
      }) {
        count += 1;
      }
    }
  }
  count
}

#[cfg(test)]
mod test {
  use crate::{
    school_schedule::SchoolSchedule,
    week_calendar::{
      TIMESLOT_09_00, TIMESLOT_10_00, TIMESLOT_11_00, TIMESLOT_12_00, TIMESLOT_13_00,
    },
  };

  use self::week_calendar::TIMESLOT_08_00;

  use super::*;

  #[test]
  fn count_outside_session_length_test() {
    let mut state = ClassCalendar::default();
    let k0 = state.new_class();
    let d0: week_calendar::Day = 0.try_into().unwrap();

    assert_eq!(count_outside_session_length(&state, 2, 4), 0);
    state
      .add_one_class(d0, week_calendar::TIMESLOT_15_00.try_into().unwrap(), k0)
      .unwrap();
    assert_eq!(count_outside_session_length(&state, 2, 4), 1);
    state
      .add_one_class(d0, week_calendar::TIMESLOT_16_00.try_into().unwrap(), k0)
      .unwrap();
    assert_eq!(count_outside_session_length(&state, 2, 4), 0);
    state
      .add_one_class(d0, week_calendar::TIMESLOT_17_00.try_into().unwrap(), k0)
      .unwrap();
    assert_eq!(count_outside_session_length(&state, 2, 4), 0);
    state
      .add_one_class(d0, week_calendar::TIMESLOT_18_00.try_into().unwrap(), k0)
      .unwrap();
    assert_eq!(count_outside_session_length(&state, 2, 4), 0);
    state
      .add_one_class(d0, week_calendar::TIMESLOT_19_00.try_into().unwrap(), k0)
      .unwrap();
    assert_eq!(count_outside_session_length(&state, 2, 4), 1);
  }

  #[test]
  fn count_inconsistent_class_timeslots_test() {
    let mut state = ClassCalendar::default();
    let k6 = state.new_class();
    let k7 = state.new_class();
    let d0: week_calendar::Day = 0.try_into().unwrap();
    let d3: week_calendar::Day = 3.try_into().unwrap();
    let d4: week_calendar::Day = 4.try_into().unwrap();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0);
    state
      .add_one_class(d0, week_calendar::TIMESLOT_18_00.try_into().unwrap(), k7)
      .unwrap();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0);
    state
      .add_one_class(d4, week_calendar::TIMESLOT_18_00.try_into().unwrap(), k6)
      .unwrap();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0);
    state
      .add_one_class(d4, week_calendar::TIMESLOT_18_00.try_into().unwrap(), k7)
      .unwrap();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0);
    state
      .add_one_class(d3, week_calendar::TIMESLOT_19_00.try_into().unwrap(), k6)
      .unwrap();
    assert_eq!(count_inconsistent_class_timeslots(&state), 2);
    state
      .add_one_class(d3, week_calendar::TIMESLOT_18_00.try_into().unwrap(), k6)
      .unwrap();
    assert_eq!(count_inconsistent_class_timeslots(&state), 1);
    state
      .add_one_class(d0, week_calendar::TIMESLOT_19_00.try_into().unwrap(), k6)
      .unwrap();
    assert_eq!(count_inconsistent_class_timeslots(&state), 0);
  }

  #[test]
  fn count_labs_on_different_days_test() {
    let mut schedule = SchoolSchedule::default();
    let p0 = schedule.add_new_professor();
    let k0 = schedule.add_new_class(p0);
    let mut class_0 = schedule.get_class_entry(k0).unwrap();
    class_0.set_hours(3);
    class_0.set_allowed_classroom_types(ClassroomType::AulaSimple | ClassroomType::AulaDoble);
    let k1 = schedule.add_new_class(p0);
    let mut class_1 = schedule.get_class_entry(k1).unwrap();
    class_1.set_allowed_classroom_types(ClassroomType::LabFisica | ClassroomType::LabQuimica);
    class_1.set_hours(3);
    let state = schedule.get_class_calendar_mut();
    let d0 = week_calendar::Day::try_from(0).unwrap();
    let d1 = week_calendar::Day::try_from(1).unwrap();
    let d2 = week_calendar::Day::try_from(2).unwrap();
    state
      .add_one_class(d0, TIMESLOT_08_00.try_into().unwrap(), k0)
      .unwrap();
    state
      .add_one_class(d1, TIMESLOT_08_00.try_into().unwrap(), k0)
      .unwrap();
    state
      .add_one_class(d2, TIMESLOT_08_00.try_into().unwrap(), k0)
      .unwrap();
    state
      .add_one_class(d0, TIMESLOT_08_00.try_into().unwrap(), k1)
      .unwrap();
    state
      .add_one_class(d0, TIMESLOT_09_00.try_into().unwrap(), k1)
      .unwrap();
    state
      .add_one_class(d0, TIMESLOT_11_00.try_into().unwrap(), k1)
      .unwrap();
    state.move_one_class(
      d0,
      TIMESLOT_09_00.try_into().unwrap(),
      d1,
      TIMESLOT_08_00.try_into().unwrap(),
      k1,
    );
  }

  #[test]
  fn test_count_incontinuous_classes() {
    let mut state = ClassCalendar::default();
    assert_eq!(count_incontinuous_classes(&state), 0);
    let d2: week_calendar::Day = 2.try_into().unwrap();
    let d3: week_calendar::Day = 3.try_into().unwrap();
    for _i in 1..=6 {
      let _ki = state.new_class();
    }
    let k7: ClassKey = state.new_class();
    let _k8 = state.new_class();
    let k9: ClassKey = state.new_class();
    state
      .add_one_class(d2, TIMESLOT_08_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 0);
    state
      .add_one_class(d2, TIMESLOT_09_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 0);
    state
      .add_one_class(d2, TIMESLOT_11_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 1);
    state
      .add_one_class(d2, TIMESLOT_13_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 1);
    state
      .add_one_class(d3, TIMESLOT_13_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 1);
    state
      .add_one_class(d3, TIMESLOT_11_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 2);
    state
      .add_one_class(d2, TIMESLOT_10_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 2);
    state
      .add_one_class(d2, TIMESLOT_12_00.try_into().unwrap(), k9)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 1);
    state
      .add_one_class(d2, TIMESLOT_10_00.try_into().unwrap(), k7)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 1);
    state
      .add_one_class(d3, TIMESLOT_11_00.try_into().unwrap(), k7)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 1);
    state
      .add_one_class(d3, TIMESLOT_09_00.try_into().unwrap(), k7)
      .unwrap();
    assert_eq!(count_incontinuous_classes(&state), 2);
  }
}
