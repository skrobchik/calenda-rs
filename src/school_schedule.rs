use std::{
  fmt::Display,
  ops::{Index, IndexMut},
};

use egui::Color32;
use enum_iterator::Sequence;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::week_calendar::{WeekCalendar, Weekday};

const MAX_CLASSES: usize = 128;
const MAX_PROFESSORS: usize = 128;

#[derive(Serialize, Deserialize, Clone, Copy)]
enum Availability {
  Prefered,
  Available,
  AvailableIfNeeded,
  NotAvailable,
}

#[derive(Serialize, Deserialize, Clone)]
struct Professor {
  availability: WeekCalendar<Availability>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ProfessorMetadata {
  name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq)]
pub enum ClassroomType {
  Single,
  Double,
  Lab,
}

impl Display for ClassroomType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ClassroomType::Single => f.write_str("Simple"),
      ClassroomType::Double => f.write_str("Doble"),
      ClassroomType::Lab => f.write_str("Laboratorio"),
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClassMetadata {
  pub name: String,
  pub color: Color32,
}

impl ClassMetadata {
  pub fn get_color(&self) -> &Color32 {
    &&self.color
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Class {
  pub professor: usize,
  pub classroom_type: ClassroomType,
  pub class_hours: u8,
}

#[derive(Serialize, Deserialize, Clone)]
struct SimulationInformation {
  #[serde(with = "BigArray")]
  classes: [Option<Class>; MAX_CLASSES],
  #[serde(with = "BigArray")]
  professors: [Option<Professor>; MAX_PROFESSORS],
}

impl Default for SimulationInformation {
  fn default() -> Self {
    const CLASSES_INIT: Option<Class> = None;
    const PROFESSORS_INIT: Option<Professor> = None;
    Self {
      classes: [CLASSES_INIT; MAX_CLASSES],
      professors: [PROFESSORS_INIT; MAX_PROFESSORS],
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Classes {
  #[serde(with = "BigArray")]
  data: [u8; MAX_CLASSES],
}

impl Default for Classes {
  fn default() -> Self {
    Self {
      data: [0; MAX_CLASSES],
    }
  }
}

impl Index<usize> for Classes {
  type Output = u8;

  fn index(&self, index: usize) -> &Self::Output {
    &self.data[index]
  }
}

impl IndexMut<usize> for Classes {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.data[index]
  }
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

#[derive(Serialize, Deserialize, Clone)]
pub struct SchoolSchedule {
  #[serde(with = "BigArray")]
  class_metadata: [Option<ClassMetadata>; MAX_CLASSES],
  #[serde(with = "BigArray")]
  professor_metadata: [Option<ProfessorMetadata>; MAX_PROFESSORS],
  simulation_information: SimulationInformation,
  schedule: WeekCalendar<Classes>,
}

impl Default for SchoolSchedule {
  fn default() -> Self {
    const CLASS_METADATA_INIT: Option<ClassMetadata> = None;
    const PROFESSOR_METADATA_INIT: Option<ProfessorMetadata> = None;
    Self {
      class_metadata: [CLASS_METADATA_INIT; MAX_CLASSES],
      professor_metadata: [PROFESSOR_METADATA_INIT; MAX_PROFESSORS],
      simulation_information: Default::default(),
      schedule: Default::default(),
    }
  }
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
  pub fn get_class_data<'a>(&'a self, day: Weekday, timeslot: usize) -> Vec<ClassData<'a>> {
    let slot: [u8; MAX_CLASSES] = self.schedule[day.into()][timeslot].into();
    let classes = &self.simulation_information.classes;
    let class_metadata = &self.class_metadata;
    slot
      .iter()
      .filter(|count| **count > 0)
      .enumerate()
      .map(|(class_id, count)| ClassData {
        count: *count,
        class: classes[class_id].as_ref().unwrap(),
        class_metadata: class_metadata[class_id].as_ref().unwrap(),
        class_id,
      })
      .collect_vec()
  }

  pub fn get_classes(&self) -> Vec<(&Class, &ClassMetadata)> {
    self
      .simulation_information
      .classes
      .iter()
      .zip(&self.class_metadata)
      .filter_map(|t| match t {
        (Some(class), Some(metadata)) => Some((class, metadata)),
        _ => None,
      })
      .collect()
  }

  pub fn get_classes_mut(&mut self) -> Vec<(&mut Class, &mut ClassMetadata, usize)> {
    let classes = &mut self.simulation_information.classes;
    let class_metadata = &mut self.class_metadata;
    classes
      .iter_mut()
      .zip(class_metadata)
      .enumerate()
      .filter_map(|t| match t {
        (i, (Some(class), Some(metadata))) => Some((class, metadata, i)),
        _ => None,
      })
      .collect()
  }

  pub fn add_new_class(&mut self) -> Option<(&mut Class, &mut ClassMetadata)> {
    let class_metadata = &mut self.class_metadata;
    let classes = &mut self.simulation_information.classes;

    let (class_id, (metadata, class)) = class_metadata
      .iter_mut()
      .zip(classes.iter_mut())
      .enumerate()
      .find(|(_i, (a, b))| {
        assert!(a.is_none() == b.is_none());
        a.is_none()
      })?;

    *metadata = Some(ClassMetadata {
      name: "New Class".to_string(),
      color: Color32::LIGHT_YELLOW,
    });
    let metadata = metadata.as_mut().unwrap();

    *class = Some(Class {
      professor: 0,
      classroom_type: ClassroomType::Single,
      class_hours: 1,
    });
    let class = class.as_mut().unwrap();

    let x = &mut self.schedule[0][0][class_id];
    *x = class.class_hours;

    Some((class, metadata))
  }
}

fn calculate_energy(simulation_information: SimulationInformation) -> f32 {
  todo!()
}
