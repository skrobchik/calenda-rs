use std::{
  collections::{hash_map::Keys, HashMap},
  hash::Hash,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RealCounter<T>
where
  T: Hash + Eq,
{
  count: HashMap<T, usize>,
  total_count: usize,
}

impl<T> RealCounter<T>
where
  T: Hash + Eq,
{
  pub fn new() -> Self {
    Default::default()
  }
  pub fn increment(&mut self, k: T) -> usize {
    let v = self.count.entry(k).or_default();
    *v += 1;
    self.total_count += 1;
    *v
  }
  pub fn decrement(&mut self, k: &T) -> Option<usize> {
    let v = self.count.get_mut(&k)?;
    *v -= 1;
    self.total_count -= 1;
    if *v == 0 {
      self.count.remove(&k);
      return Some(0);
    }
    Some(*v)
  }
  pub fn remove(&mut self, k: &T) -> Option<usize> {
    match self.count.remove(k) {
      Some(v) => {
        self.total_count -= v;
        Some(v)
      }
      None => None,
    }
  }
  pub fn clear(&mut self) {
    self.total_count = 0;
    self.count.clear()
  }
  pub fn get_count(&self, k: &T) -> usize {
    let v = self.count.get(&k);
    if v.is_none() {
      return 0;
    }
    *v.unwrap()
  }
  pub fn is_positive(&self, k: &T) -> bool {
    self.count.contains_key(&k)
  }
  pub fn count_unique(&self) -> usize {
    self.count.len()
  }
  pub fn count_total(&self) -> usize {
    self.total_count
  }
  pub fn iter(&self) -> std::collections::hash_map::Iter<'_, T, usize> {
    self.count.iter()
  }
  pub fn is_empty(&self) -> bool {
    self.count.is_empty()
  }
  pub fn keys(&self) -> Keys<'_, T, usize> {
    self.count.keys()
  }
}

impl<T> Default for RealCounter<T>
where
  T: Hash + Eq,
{
  fn default() -> Self {
    Self {
      count: Default::default(),
      total_count: Default::default(),
    }
  }
}

impl<T> FromIterator<(T, usize)> for RealCounter<T>
where
  T: Hash + Eq,
{
  fn from_iter<I: IntoIterator<Item = (T, usize)>>(iter: I) -> Self {
    let mut c = RealCounter::new();
    for i in iter {
      c.total_count += i.1;
      let v = c.count.entry(i.0).or_default();
      *v = i.1;
    }
    c
  }
}
