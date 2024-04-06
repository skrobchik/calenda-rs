use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;

use crate::ClassKey;

use crate::ProfessorKey;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct ScheduleMetadata {
  pub(super) professors: SecondaryMap<ProfessorKey, ProfessorMetadata>,
  pub(super) classes: SecondaryMap<ClassKey, ClassMetadata>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfessorMetadata {
  pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClassMetadata {
  pub name: String,
  pub rgba: [u8; 4],
  pub class_code: String,
}
