use serde::{Deserialize, Serialize};

use crate::school_schedule::{Semester, SimulationConstraints};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) enum ClassFilter {
    None,
    Semester(Semester),
    ProfessorId(usize),
}

impl ClassFilter {
    pub(crate) fn filter(&self, class_id: usize, simulation_constraints: &SimulationConstraints) -> bool {
        match self {
            ClassFilter::None => true,
            ClassFilter::Semester(s) => simulation_constraints.get_classes().get(class_id).unwrap().get_semester() == s,
            ClassFilter::ProfessorId(p) => simulation_constraints.get_classes().get(class_id).unwrap().get_professor_id() == p,
        }
    }
}