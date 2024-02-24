use serde::{Deserialize, Serialize};

use crate::school_schedule::{class_calendar::ClassId, Semester, SimulationConstraints};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) enum ClassFilter {
  #[default]
  None,
  Semester(Semester),
  ProfessorId(usize),
}

impl ClassFilter {
  pub(crate) fn filter(
    &self,
    class_id: ClassId,
    simulation_constraints: &SimulationConstraints,
  ) -> bool {
    match self {
      ClassFilter::None => true,
      ClassFilter::Semester(s) => {
        if let Some(class) = simulation_constraints.get_class(class_id) {
          class.get_semester() == s
        } else {
          false
        }
      }
      ClassFilter::ProfessorId(p) => {
        if let Some(class) = simulation_constraints.get_class(class_id) {
          class.get_professor_id() == p
        } else {
          false
        }
      }
    }
  }
}
