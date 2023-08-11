use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(super) struct ScheduleMetadata {
  pub(super) professors: Vec<ProfessorMetadata>,
  pub(super) classes: Vec<ClassMetadata>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfessorMetadata {
  pub name: String,
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
