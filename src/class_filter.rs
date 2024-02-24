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
        simulation_constraints
          .get_classes()
          .get(usize::from(class_id))
          .unwrap()
          .get_semester()
          == s
      }
      ClassFilter::ProfessorId(p) => {
        simulation_constraints
          .get_classes()
          .get(usize::from(class_id))
          .unwrap()
          .get_professor_id()
          == p
      }
    }
  }
}
