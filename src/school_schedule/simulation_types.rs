use std::fmt::Display;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

use crate::week_calendar::WeekCalendar;

use anyhow::anyhow;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct SimulationConstraints {
  pub(super) classes: Vec<Class>,
  pub(super) professors: Vec<Professor>,
}

impl SimulationConstraints {
  pub(crate) fn get_classes(&self) -> &Vec<Class> {
    &self.classes
  }
  pub(crate) fn get_professors(&self) -> &Vec<Professor> {
    &self.professors
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub(crate) struct Class {
  pub(super) professor_id: usize,
  pub(super) classroom_type: ClassroomType,
  pub(super) class_hours: u8,
  pub(super) semester: Semester,
  pub(super) group: Group,
}

impl Class {
  pub(crate) fn get_professor_id(&self) -> &usize {
    &self.professor_id
  }
  pub(crate) fn get_classroom_type(&self) -> &ClassroomType {
    &self.classroom_type
  }
  pub(crate) fn get_class_hours(&self) -> &u8 {
    &self.class_hours
  }
  pub(crate) fn get_semester(&self) -> &Semester {
    &self.semester
  }
  pub(crate) fn get_group(&self) -> &Group {
    &self.group
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Professor {
  pub(crate) availability: WeekCalendar<Availability>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq, Debug)]
pub(crate) enum Semester {
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

impl From<Semester> for u32 {
  fn from(val: Semester) -> Self {
    (&val).into()
  }
}

impl From<&Semester> for u32 {
  fn from(val: &Semester) -> Self {
    match val {
      Semester::S1 => 1,
      Semester::S2 => 2,
      Semester::S3 => 3,
      Semester::S4 => 4,
      Semester::S5 => 5,
      Semester::S6 => 6,
      Semester::S7 => 7,
      Semester::S8 => 8,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq, Debug)]
pub(crate) enum Group {
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
pub(crate) enum Availability {
  Available,
  AvailableIfNeeded,
  #[default]
  NotAvailable,
}

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq, Debug)]
pub(crate) enum ClassroomType {
  AulaSimple,
  AulaDoble,
  LabQuimica,
  LabFisica,
  AulaComputo,
}

impl Display for ClassroomType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      ClassroomType::AulaSimple => "Aula Simple",
      ClassroomType::AulaDoble => "Aula Doble",
      ClassroomType::LabQuimica => "Lab Quimica",
      ClassroomType::LabFisica => "Lab Fisica",
      ClassroomType::AulaComputo => "Aula Computo",
    })
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum Classroom {
  Aula1,
  Aula2_3,
  Aula4,
  Aula5_6,
  SalaSeminarios,
  SalaComputo,
  LabFisica,
  LabQuimica,
}

impl Classroom {
  pub(crate) fn get_type(self) -> ClassroomType {
    match self {
      Classroom::Aula1 => ClassroomType::AulaSimple,
      Classroom::Aula2_3 => ClassroomType::AulaDoble,
      Classroom::Aula4 => ClassroomType::AulaSimple,
      Classroom::Aula5_6 => ClassroomType::AulaDoble,
      Classroom::SalaSeminarios => ClassroomType::AulaDoble,
      Classroom::SalaComputo => ClassroomType::AulaComputo,
      Classroom::LabFisica => ClassroomType::LabFisica,
      Classroom::LabQuimica => ClassroomType::LabQuimica,
    }
  }
}
