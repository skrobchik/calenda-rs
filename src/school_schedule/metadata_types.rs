use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct ScheduleMetadata {
  pub(super) professors: Vec<ProfessorMetadata>,
  pub(super) classes: Vec<ClassMetadata>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ProfessorMetadata {
  pub(crate) name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ClassMetadata {
  pub(crate) name: String,
  pub(crate) color: Color32,
}
