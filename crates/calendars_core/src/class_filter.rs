use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
  school_schedule::{
    class_calendar::ClassKey, Classroom, ClassroomAssignmentKey, ProfessorKey, Semester,
    SimulationConstraints,
  },
  simulation::assign_classrooms,
  ClassCalendar, Day, Timeslot,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]

pub enum ClassFilter {
  #[default]
  None,
  Semester(Semester),
  Professor(ProfessorKey),
  Classroom(Classroom),
}

static mut CLASSROOM_ASSIGNMENT_MEMO: Lazy<BTreeMap<ClassroomAssignmentKey, Classroom>> =
  Lazy::new(BTreeMap::new);

impl ClassFilter {
  pub fn filter(
    &self,
    class_key: ClassKey,
    simulation_constraints: &SimulationConstraints,
    state: &ClassCalendar,
    day: Day,
    timeslot: Timeslot,
    regenerate_memo: bool, // TODO: I know this is horrible code,
                           // but I have a presentation tomorrow
                           // and there's no obvious way to make this not super slow in the UI.
  ) -> bool {
    match self {
      ClassFilter::None => true,
      ClassFilter::Semester(s) => {
        simulation_constraints
          .get_class(class_key)
          .unwrap()
          .get_semester()
          == s
      }
      ClassFilter::Professor(p) => {
        simulation_constraints
          .get_class(class_key)
          .unwrap()
          .get_professor_id()
          == *p
      }
      ClassFilter::Classroom(c) => {
        // TODO: Regenerating classroom assignment each time is slow.
        // For now this is only used in export and when showing the calendar
        // so it's ok. If this filter is used in the simulation, might be very bad.
        // ^^^
        // actually super slow in UI too, need to think about how to refactor this.
        if regenerate_memo {
          unsafe {
            *Lazy::force_mut(&mut CLASSROOM_ASSIGNMENT_MEMO) =
              assign_classrooms(state, simulation_constraints);
          }
        };
        let classroom_assignment = unsafe { Lazy::force(&CLASSROOM_ASSIGNMENT_MEMO) };
        let key = ClassroomAssignmentKey {
          day,
          timeslot,
          class_key,
        };
        if state.get_count(day, timeslot, class_key) == 0 {
          return false;
        }
        classroom_assignment.get(&key).unwrap() == c
      }
    }
  }
}
