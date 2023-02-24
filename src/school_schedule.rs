use itertools::Itertools;

use crate::week_calendar::{GetDay, WeekCalendar, Weekday};

const MAX_CLASSES: usize = 128;
const MAX_PROFESSORS: usize = 128;

enum Availability {
  Prefered,
  Available,
  AvailableIfNeeded,
  NotAvailable,
}

struct Professor {
  availability: WeekCalendar<Availability>,
}

struct ProfessorMetadata {
  name: String,
}

enum ClassroomType {
  Single,
  Double,
  Lab,
}

struct ClassMetadata {
  name: String,
}

struct Class {
  professor: usize,
  classroom_type: ClassroomType,
  class_hours: u8,
}

struct SimulationInformation {
  classes: [Option<Class>; MAX_CLASSES],
  professors: [Option<Professor>; MAX_PROFESSORS],
}

pub struct SchoolSchedule {
  class_metadata: [Option<ClassMetadata>; MAX_CLASSES],
  professor_metadata: [Option<ProfessorMetadata>; MAX_PROFESSORS],
  simulation_information: SimulationInformation,
  schedule: WeekCalendar<[u8; MAX_CLASSES]>,
}

fn generate_schedule(
  simulation_information: SimulationInformation,
) -> WeekCalendar<[u8; MAX_CLASSES]> {
  todo!()
}

pub struct ClassData<'a> {
  pub count: u8,
  pub class_id: usize,
  pub class: &'a Class,
  pub class_metadata: &'a ClassMetadata,
}

impl SchoolSchedule {
  pub fn get_classes<'a>(&self, day: Weekday, timeslot: usize) -> Vec<ClassData<'a>> {
    let slot = self.schedule.get_day(&day)[timeslot];
    let classes = &self.simulation_information.classes;
    slot
      .iter()
      .enumerate()
      .map(|(class_id, count)| ClassData {
        count: *count,
        class: &self.simulation_information.classes[class_id].unwrap(),
        class_metadata: &self.class_metadata[class_id].unwrap(),
        class_id,
      })
      .collect_vec()
  }
}

fn calculate_energy(simulation_information: SimulationInformation) -> f32 {
  todo!()
}
