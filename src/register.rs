use std::collections::BTreeMap;

pub type RegisterId = u64;

pub struct Register<T> {
  unique_count: RegisterId,
  entries: BTreeMap<RegisterId, T>,
}

impl<T> Register<T> {
  fn add_entry(&mut self, entry: T) -> RegisterId {
    let id = self.unique_count;
    self.unique_count += 1;
    self.entries.insert(id, entry);
    id
  }
  fn remove_entry(&mut self, id: &RegisterId) -> Option<T> {
    self.entries.remove(id)
  }
  fn get_entry(&self, id: &RegisterId) -> Option<&T> {
    self.entries.get(id)
  }
  fn get_entry_mut(&mut self, id: &RegisterId) -> Option<&mut T> {
    self.entries.get_mut(id)
  }
}

impl<T> Default for Register<T> {
  fn default() -> Self {
    Self {
      unique_count: Default::default(),
      entries: Default::default(),
    }
  }
}
