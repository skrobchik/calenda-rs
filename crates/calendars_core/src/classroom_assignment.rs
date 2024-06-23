use std::{collections::BTreeMap, rc::Rc};

use itertools::Itertools;
use strum::{IntoEnumIterator, VariantArray};

use crate::{
  week_calendar, ClassCalendar, ClassKey, Classroom, ClassroomAssignmentKey, ClassroomType,
  OptimizationConstraints,
};

pub(crate) fn assign_classrooms(
  state: &ClassCalendar,
  constraints: &OptimizationConstraints,
) -> BTreeMap<ClassroomAssignmentKey, Classroom> {
  let matching = assign_classrooms_matching(state, constraints);
  matching.collect()
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
enum ClassroomAssignmentVertex {
  Class(ClassKey),
  Classroom(Classroom),
}

fn timeslot_assign_classrooms<'a>(
  state: &'a ClassCalendar,
  constraints: &'a OptimizationConstraints,
  day: week_calendar::Day,
  timeslot: week_calendar::Timeslot,
  available_classrooms: Rc<[Vec<Classroom>; ClassroomType::VARIANTS.len()]>,
) -> impl Iterator<Item = (ClassroomAssignmentKey, Classroom)> + 'a {
  let mut edges: Vec<(ClassroomAssignmentVertex, ClassroomAssignmentVertex)> = Vec::new();
  for class_key in state
    .iter_class_keys()
    .filter(|k| state.get_count(day, timeslot, *k) > 0)
  {
    let class = constraints.classes.get(class_key).unwrap();
    let classrooms = class
      .allowed_classroom_types
      .iter()
      .map(|classroom_type| {
        ClassroomType::VARIANTS
          .iter()
          .position(|v| *v == classroom_type)
          .unwrap()
      })
      .flat_map(|classroom_type_i| &available_classrooms[classroom_type_i])
      .unique();
    for classroom in classrooms {
      edges.push((
        ClassroomAssignmentVertex::Class(class_key),
        ClassroomAssignmentVertex::Classroom(*classroom),
      ));
    }
  }
  let matching = hopcroft_karp::matching(&edges);
  matching.into_iter().map(move |(a, b)| match (a, b) {
    (
      ClassroomAssignmentVertex::Class(class_key),
      ClassroomAssignmentVertex::Classroom(classroom),
    ) => (
      ClassroomAssignmentKey {
        day,
        timeslot,
        class_key,
      },
      classroom,
    ),
    _ => unreachable!(),
  })
}

fn iter_week() -> impl Iterator<Item = (week_calendar::Day, week_calendar::Timeslot)> {
  week_calendar::Day::all().flat_map(|d| week_calendar::Timeslot::all().map(move |t| (d, t)))
}

fn assign_classrooms_matching<'a>(
  state: &'a ClassCalendar,
  constraints: &'a OptimizationConstraints,
) -> impl Iterator<Item = (ClassroomAssignmentKey, Classroom)> + 'a {
  let available_classrooms: [Vec<Classroom>; ClassroomType::VARIANTS.len()] =
    ClassroomType::VARIANTS
      .iter()
      .map(|v| {
        Classroom::iter()
          .filter(|c| c.get_type() == *v)
          .collect_vec()
      })
      .collect_vec()
      .try_into()
      .unwrap();
  let available_classrooms = Rc::new(available_classrooms);

  iter_week().flat_map(move |(day, timeslot)| {
    timeslot_assign_classrooms(
      state,
      constraints,
      day,
      timeslot,
      available_classrooms.clone(),
    )
  })
}

pub(crate) fn count_classroom_assignment_collisions(
  state: &ClassCalendar,
  constraints: &OptimizationConstraints,
) -> usize {
  state
    .class_entries()
    .len()
    .checked_sub(assign_classrooms_matching(state, constraints).count())
    .expect("Can't be more matching than class entries")
}
