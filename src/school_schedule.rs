use egui::Color32;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::week_calendar::{GetDay, WeekCalendar, Weekday};

const MAX_CLASSES: usize = 128;
const MAX_PROFESSORS: usize = 128;

#[derive(Serialize, Deserialize)]
enum Availability {
  Prefered,
  Available,
  AvailableIfNeeded,
  NotAvailable,
}

#[derive(Serialize, Deserialize)]
struct Professor {
  availability: WeekCalendar<Availability>,
}

#[derive(Serialize, Deserialize)]
struct ProfessorMetadata {
  name: String,
}

#[derive(Serialize, Deserialize)]
enum ClassroomType {
  Single,
  Double,
  Lab,
}

#[derive(Serialize, Deserialize)]
pub struct ClassMetadata {
  name: String,
  color: Color32,
}

#[derive(Serialize, Deserialize)]
pub struct Class {
  professor: usize,
  classroom_type: ClassroomType,
  class_hours: u8,
}

#[derive(Serialize, Deserialize)]
struct SimulationInformation {
  #[serde(with = "BigArray")]
  classes: [Option<Class>; MAX_CLASSES],
  #[serde(with = "BigArray")]
  professors: [Option<Professor>; MAX_PROFESSORS],
}

#[derive(Serialize, Deserialize)]
struct Classes {
  #[serde(with = "BigArray")]
  data: [u8; MAX_CLASSES],
}

impl Into<[u8; MAX_CLASSES]> for Classes {
  fn into(self) -> [u8; MAX_CLASSES] {
    self.data
  }
}

impl From<[u8; MAX_CLASSES]> for Classes {
  fn from(data: [u8; MAX_CLASSES]) -> Self {
    Classes { data }
  }
}

#[derive(Serialize, Deserialize)]
pub struct SchoolSchedule {
  #[serde(with = "BigArray")]
  class_metadata: [Option<ClassMetadata>; MAX_CLASSES],
  #[serde(with = "BigArray")]
  professor_metadata: [Option<ProfessorMetadata>; MAX_PROFESSORS],
  simulation_information: SimulationInformation,
  schedule: WeekCalendar<Classes>,
}

fn generate_schedule(simulation_information: SimulationInformation) -> WeekCalendar<Classes> {
  todo!()
}

pub struct ClassData<'a> {
  pub count: u8,
  pub class_id: usize,
  pub class: &'a Class,
  pub class_metadata: &'a ClassMetadata,
}

impl SchoolSchedule {
  pub fn get_classes<'a>(&'a self, day: Weekday, timeslot: usize) -> Vec<ClassData<'a>> {
    let slot: [u8; MAX_CLASSES] = self.schedule.get_day(&day)[timeslot].into();
    let classes = &self.simulation_information.classes;
    let class_metadata = &self.class_metadata;
    slot
      .iter()
      .enumerate()
      .map(|(class_id, count)| ClassData {
        count: *count,
        class: classes[class_id].as_ref().unwrap(),
        class_metadata: class_metadata[class_id].as_ref().unwrap(),
        class_id,
      })
      .collect_vec()
  }
}

fn calculate_energy(simulation_information: SimulationInformation) -> f32 {
  todo!()
}
