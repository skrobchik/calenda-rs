use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub enum ClassRoomType {
  SmallClassroom,
  BigClassroom,
  PhysicsLab,
  ChemistryLab,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum SemesterNumber {
  S1,
  S2,
  S3,
  S4,
  S5,
  S6,
  S7,
  S8,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ClassMetadata {
  pub name: String,
  pub classroom_type: ClassRoomType,
  pub semester_number: SemesterNumber,
  pub professor_id: usize,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ProfessorMetadata {
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MetadataRegister {
  class_register: Vec<ClassMetadata>,
  professor_register: Vec<ProfessorMetadata>,
}

impl Default for MetadataRegister {
  fn default() -> Self {
    Self {
      class_register: Default::default(),
      professor_register: Default::default(),
    }
  }
}

impl MetadataRegister {
  pub fn get_class_metadata(&self, class_id: usize) -> Option<&ClassMetadata> {
    self.class_register.get(class_id)
  }
  pub fn get_professor_metadata(&self, professor_id: usize) -> Option<&ProfessorMetadata> {
    self.professor_register.get(professor_id)
  }
  pub fn get_class_metadata_mut(&mut self, class_id: usize) -> Option<&mut ClassMetadata> {
    self.class_register.get_mut(class_id)
  }
  pub fn get_professor_metadata_mut(
    &mut self,
    professor_id: usize,
  ) -> Option<&mut ProfessorMetadata> {
    self.professor_register.get_mut(professor_id)
  }
  pub fn professor_register_len(&self) -> usize {
    self.professor_register.len()
  }
  pub fn class_register_len(&self) -> usize {
    self.class_register.len()
  }
  pub fn add_professor(&mut self, metadata: ProfessorMetadata) -> usize {
    self.professor_register.push(metadata);
    self.professor_register.len() - 1
  }
  pub fn add_class(&mut self, metadata: ClassMetadata) -> usize {
    self.class_register.push(metadata);
    self.class_register.len() - 1
  }
  pub fn get_professor_list(&self) -> &Vec<ProfessorMetadata> {
    &self.professor_register
  }
}
