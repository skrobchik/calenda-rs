use std::{
  fmt::Display,
  ops::{Index, IndexMut},
};

use egui::Color32;
use enum_iterator::Sequence;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::{
  timeslot::{DAY_RANGE, TIMESLOT_RANGE},
  week_calendar::{WeekCalendar, Weekday},
};

const MAX_CLASSES: usize = 1046;
const MAX_PROFESSORS: usize = 128;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub enum Availability {
  Available,
  AvailableIfNeeded,
  #[default]
  NotAvailable,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Professor {
  pub availability: WeekCalendar<Availability>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfessorMetadata {
  pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClassMetadata {
  pub name: String,
  pub color: Color32,
}

impl ClassMetadata {
  pub fn get_color(&self) -> &Color32 {
    &self.color
  }

  pub fn get_name(&self) -> &str {
    &self.name
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Class {
  pub professor: usize,
  pub classroom_type: ClassroomType,
  pub class_hours: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimulationConstraints {
  pub classes: Vec<Option<Class>>,
  pub professors: Vec<Option<Professor>>,
}

impl Default for SimulationConstraints {
  fn default() -> Self {
    const CLASSES_INIT: Option<Class> = None;
    const PROFESSORS_INIT: Option<Professor> = None;
    Self {
      classes: vec![CLASSES_INIT; MAX_CLASSES],
      professors: vec![PROFESSORS_INIT; MAX_PROFESSORS],
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Classes {
  data: Vec<u8>,
}

impl Default for Classes {
  fn default() -> Self {
    Self {
      data: std::iter::repeat(0).take(MAX_CLASSES).collect(),
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

impl Into<Vec<u8>> for Classes {
    fn into(self) -> Vec<u8> {
        self.data
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SchoolSchedule {
  #[serde(with = "BigArray")]
  pub class_metadata: [Option<ClassMetadata>; MAX_CLASSES],
  #[serde(with = "BigArray")]
  pub professor_metadata: [Option<ProfessorMetadata>; MAX_PROFESSORS],
  pub simulation_information: SimulationConstraints,
  pub schedule: WeekCalendar<Classes>,
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

pub struct ClassData<'a> {
  pub count: u8,
  pub class_id: usize,
  pub class: &'a Class,
  pub class_metadata: &'a ClassMetadata,
}

impl SchoolSchedule {
  pub fn get_class_data(&self, day: Weekday, timeslot: usize) -> Vec<ClassData<'_>> {
    let slot: Vec<u8> = self.schedule.get(day, timeslot).unwrap().clone().into();
    let classes = &self.simulation_information.classes;
    let class_metadata = &self.class_metadata;
    slot
      .iter()
      .enumerate()
      .filter(|(_class_id, count)| **count > 0)
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

  pub fn get_classes_and_professors_mut(
    &mut self,
  ) -> (
    Vec<(&mut Class, &mut ClassMetadata, usize)>,
    Vec<(&mut Professor, &mut ProfessorMetadata, usize)>,
  ) {
    let classes = &mut self.simulation_information.classes;
    let class_metadata = &mut self.class_metadata;
    let professors = &mut self.simulation_information.professors;
    let professor_metadata = &mut self.professor_metadata;
    let classes = classes
      .iter_mut()
      .zip(class_metadata)
      .enumerate()
      .filter_map(|t| match t {
        (i, (Some(class), Some(metadata))) => Some((class, metadata, i)),
        _ => None,
      })
      .collect();
    let professors = professors
      .iter_mut()
      .zip(professor_metadata)
      .enumerate()
      .filter_map(|t| match t {
        (i, (Some(professor), Some(metadata))) => Some((professor, metadata, i)),
        _ => None,
      })
      .collect();
    (classes, professors)
  }

  pub fn get_classes_mut(&mut self) -> Vec<(&mut Class, &mut ClassMetadata, usize)> {
    self.get_classes_and_professors_mut().0
  }

  pub fn get_professors_mut(&mut self) -> Vec<(&mut Professor, &mut ProfessorMetadata, usize)> {
    self.get_classes_and_professors_mut().1
  }

  fn add_hours_to_schedule(&mut self, class_id: usize, count: u8) {
  self.schedule.get_mut(0_usize, 0_usize).unwrap()[class_id] = self.schedule.get(0_usize, 0_usize).unwrap()[class_id].checked_add(count).unwrap();
  }

  /// Attempts to remove `count` instances of class with `class_id` from schedule.
  /// Returns the amount that was left to remove.
  /// If `count` instances were removed succesfully, return value is 0.
  /// If there are not enough classes to remove, return the amount that was left to remove.
  fn remove_hours_from_schedule(&mut self, class_id: usize, count: u8) -> u8 {
    let mut count = count;
    for day in DAY_RANGE {
      for timeslot in TIMESLOT_RANGE {
        let dc = count.min(self.schedule.get(day, timeslot).unwrap()[class_id]);
        count -= dc;
        self.schedule.get_mut(day, timeslot).unwrap()[class_id] -= dc;
        if count == 0 {
          return count;
        }
      }
    }
    count
  }

  pub(crate) fn fill_classes(&mut self) {
    let mut schedule_hour_count: [u8; MAX_CLASSES] = [0; MAX_CLASSES];
    for day in DAY_RANGE {
      for timeslot in TIMESLOT_RANGE {
        for class_id in 0..MAX_CLASSES {
          schedule_hour_count[class_id] += self.schedule.get(day, timeslot).unwrap()[class_id];
        }
      }
    }
    let mut classes_hour_count: [u8; MAX_CLASSES] = [0; MAX_CLASSES];
    for class_id in 0..MAX_CLASSES {
      if let Some(class) = self.simulation_information.classes[class_id] {
        classes_hour_count[class_id] = class.class_hours;
      }
    }
    for class_id in 0..MAX_CLASSES {
      let schedule_hours = schedule_hour_count[class_id];
      let class_hours = classes_hour_count[class_id];
      match Ord::cmp(&schedule_hours, &class_hours) {
        std::cmp::Ordering::Less => {
          println!(
            "Deficit of {} classes with id {} in schedule",
            class_hours - schedule_hours,
            class_id
          );
          self.add_hours_to_schedule(class_id, class_hours - schedule_hours);
        }
        std::cmp::Ordering::Equal => (),
        std::cmp::Ordering::Greater => {
          println!(
            "Excess of {} classes with id {} in schedule",
            schedule_hours - class_hours,
            class_id
          );
          self.remove_hours_from_schedule(class_id, schedule_hours - class_hours);
        }
      }
    }
  }

  pub fn add_new_professor(&mut self) -> Option<(&mut Professor, &mut ProfessorMetadata, usize)> {
    let professor_metadata = &mut self.professor_metadata;
    let professors = &mut self.simulation_information.professors;

    let (professor_id, (metadata, professor)) = professor_metadata
      .iter_mut()
      .zip(professors.iter_mut())
      .enumerate()
      .find(|(_i, (a, b))| {
        assert!(a.is_none() == b.is_none());
        a.is_none()
      })?;

    *metadata = Some(ProfessorMetadata {
      name: "New Professor".to_string(),
    });
    let metadata = metadata.as_mut().unwrap();

    *professor = Some(Professor {
      availability: WeekCalendar::default(),
    });
    let professor = professor.as_mut().unwrap();

    Some((professor, metadata, professor_id))
  }

  pub fn add_new_class(&mut self) -> Option<(&mut Class, &mut ClassMetadata)> {
    let class_metadata = &mut self.class_metadata;
    let classes = &mut self.simulation_information.classes;

    let (_class_id, (metadata, class)) = class_metadata
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
    Some((class, metadata))
  }
}
