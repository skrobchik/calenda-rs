use std::fmt::Display;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

use crate::week_calendar::WeekCalendar;

use anyhow::anyhow;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(Default)]
pub struct SimulationConstraints {
  pub classes: Vec<Class>,
  pub professors: Vec<Professor>,
}



#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Class {
  pub professor: usize,
  pub classroom_type: ClassroomType,
  pub class_hours: u8,
  pub semester: Semester,
  pub group: Group,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Professor {
  pub availability: WeekCalendar<Availability>,
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

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq, Debug)]
pub enum Semester {
  S1,
  S2,
  S3,
  S4,
  S5,
  S6,
  S7,
  S8,
}

impl Display for Semester {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Semester::S1 => f.write_str("01"),
      Semester::S2 => f.write_str("02"),
      Semester::S3 => f.write_str("03"),
      Semester::S4 => f.write_str("04"),
      Semester::S5 => f.write_str("05"),
      Semester::S6 => f.write_str("06"),
      Semester::S7 => f.write_str("07"),
      Semester::S8 => f.write_str("08"),
    }
  }
}

impl TryFrom<u32> for Semester {
  type Error = anyhow::Error;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Semester::S1),
      2 => Ok(Semester::S2),
      3 => Ok(Semester::S3),
      4 => Ok(Semester::S4),
      5 => Ok(Semester::S5),
      6 => Ok(Semester::S6),
      7 => Ok(Semester::S7),
      8 => Ok(Semester::S8),
      _ => Err(anyhow!("Invalid semester")),
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq, Debug)]
pub enum Group {
  G1,
  G2,
  G3,
  G4,
}

impl TryFrom<u32> for Group {
  type Error = anyhow::Error;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Group::G1),
      2 => Ok(Group::G2),
      3 => Ok(Group::G3),
      4 => Ok(Group::G4),
      _ => Err(anyhow!("Invalid group")),
    }
  }
}

impl Display for Group {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Group::G1 => f.write_str("01"),
      Group::G2 => f.write_str("02"),
      Group::G3 => f.write_str("03"),
      Group::G4 => f.write_str("04"),
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Availability {
  Available,
  AvailableIfNeeded,
  #[default]
  NotAvailable,
}
