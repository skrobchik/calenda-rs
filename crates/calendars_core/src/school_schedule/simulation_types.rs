use std::fmt::Display;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::week_calendar::WeekCalendar;

use super::class_calendar::ClassKey;

new_key_type! {
  pub struct ProfessorKey;
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SimulationConstraints {
  pub(super) classes: SecondaryMap<ClassKey, Class>,
  pub(super) professors: SlotMap<ProfessorKey, Professor>,
}

impl SimulationConstraints {
  pub fn get_class(&self, class_key: ClassKey) -> Option<&Class> {
    self.classes.get(class_key)
  }
  pub fn iter_professor_keys(&self) -> impl Iterator<Item = ProfessorKey> + '_ {
    self.professors.keys()
  }
  pub fn get_professors(&self) -> &SlotMap<ProfessorKey, Professor> {
    &self.professors
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Class {
  pub(super) professor_key: ProfessorKey,
  pub(super) classroom_type: ClassroomType,
  pub(super) class_hours: u8,
  pub(super) semester: Semester,
  pub(super) group: Group,
  pub(super) optative: bool,
}

impl Class {
  pub fn get_professor_id(&self) -> ProfessorKey {
    self.professor_key
  }
  pub fn get_classroom_type(&self) -> &ClassroomType {
    &self.classroom_type
  }
  pub fn get_class_hours(&self) -> &u8 {
    &self.class_hours
  }
  pub fn get_semester(&self) -> &Semester {
    &self.semester
  }
  pub fn get_group(&self) -> &Group {
    &self.group
  }
  pub fn is_optative(&self) -> bool {
    self.optative
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Professor {
  pub availability: WeekCalendar<Availability>,
  pub priority: f32,
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

#[derive(thiserror::Error, Debug)]
#[error("Invalid Semester")]
pub struct InvalidSemesterError {}

impl TryFrom<u32> for Semester {
  type Error = InvalidSemesterError;

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
      _ => Err(InvalidSemesterError {}),
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
pub enum Group {
  G1,
  G2,
  G3,
  G4,
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid Group")]
pub struct InvalidGroupError {}

impl TryFrom<u32> for Group {
  type Error = InvalidGroupError;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Group::G1),
      2 => Ok(Group::G2),
      3 => Ok(Group::G3),
      4 => Ok(Group::G4),
      _ => Err(InvalidGroupError {}),
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

#[derive(Serialize, Deserialize, Clone, Copy, Sequence, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum ClassroomType {
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

#[derive(Serialize, Deserialize, Debug, Clone, Sequence, PartialEq, Eq, PartialOrd, Ord)]
pub enum Classroom {
  Aula1,
  Aula2_3,
  Aula4,
  Aula5_6,
  SalaSeminarios,
  SalaComputo,
  LabFisica,
  LabQuimica,
}

impl Display for Classroom {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Classroom::Aula1 => "Aula 1",
      Classroom::Aula2_3 => "Aula 2-3",
      Classroom::Aula4 => "Aula 4",
      Classroom::Aula5_6 => "Aula 5-6",
      Classroom::SalaSeminarios => "Sala de Seminarios",
      Classroom::SalaComputo => "Sala de Computo",
      Classroom::LabFisica => "Lab de Fisica",
      Classroom::LabQuimica => "Lab de Quimica",
    })
  }
}

impl Classroom {
  pub fn get_type(&self) -> ClassroomType {
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
