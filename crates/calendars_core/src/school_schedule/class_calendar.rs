use crate::week_calendar;
use crate::week_calendar::WeekCalendar;
use serde::Deserialize;
use serde::Serialize;
use slotmap::new_key_type;
use slotmap::SlotMap;

new_key_type! {
  pub struct ClassKey;
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct SingleClassEntry {
  pub day: week_calendar::Day,
  pub timeslot: week_calendar::Timeslot,
  pub class_key: ClassKey,
}

#[derive(thiserror::Error, Debug)]
pub enum MoveOneClassRandomError {
  #[error("Tried to move a class randomly, but the destination was full")]
  RandomChosenDestinationFull,
  #[error("The calendar is empty (the total class count is 0)")]
  NoClassesToMove,
}

#[derive(thiserror::Error, Debug)]
pub enum AddOneClassError {
  #[error("The destination is full. Cannot increase class count.")]
  DestinationFull,
}

#[derive(thiserror::Error, Debug)]
pub enum RemoveOneClassError {
  #[error("The source is empty. Cannot decrease class count.")]
  SourceEmpty,
}

#[derive(thiserror::Error, Debug)]
pub enum RemoveOneClassAnywhereError {
  #[error("The total count for the given class in the calendar is zero.")]
  NoClasses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassEntryDelta {
  pub class_key: ClassKey,
  pub src_day: week_calendar::Day,
  pub src_timeslot: week_calendar::Timeslot,
  pub dst_day: week_calendar::Day,
  pub dst_timeslot: week_calendar::Timeslot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassCalendar {
  data: SlotMap<ClassKey, WeekCalendar<u8>>,
  class_entries: Vec<SingleClassEntry>,
}

impl ClassCalendar {
  pub(crate) fn get_entries(&self) -> &Vec<SingleClassEntry> {
    &self.class_entries
  }

  pub fn iter_class_keys<'a>(&'a self) -> impl Iterator<Item = ClassKey> + 'a {
    self.data.keys()
  }

  fn get_calendar(&self, class_key: ClassKey) -> &WeekCalendar<u8> {
    self
      .data
      .get(class_key)
      .expect("ClassKeys are always valid because we don't expose class deletion.")
  }

  fn get_calendar_mut(&mut self, class_key: ClassKey) -> &mut WeekCalendar<u8> {
    self
      .data
      .get_mut(class_key)
      .expect("ClassKeys are always valid because we don't expose class deletion.")
  }

  pub fn get_count(
    &self,
    day: week_calendar::Day,
    timeslot: week_calendar::Timeslot,
    class_key: ClassKey,
  ) -> u8 {
    let calendar = self.get_calendar(class_key);
    *calendar.get(day, timeslot)
  }

  fn move_one_class_random_delta<R: rand::Rng>(
    &self,
    rng: &mut R,
  ) -> Result<(ClassEntryDelta, usize), MoveOneClassRandomError> {
    if self.class_entries.is_empty() {
      return Err(MoveOneClassRandomError::NoClassesToMove);
    }
    let entry_index = rng.gen_range(0..self.class_entries.len());
    let entry = self.class_entries.get(entry_index).unwrap();
    let class_key = entry.class_key;
    let src_day = entry.day;
    let src_timeslot = entry.timeslot;
    let dst_day = week_calendar::Day::new_random(rng);
    let dst_timeslot = week_calendar::Timeslot::new_random(rng);
    self
      .get_count(dst_day, dst_timeslot, class_key)
      .checked_add(1)
      .and(Some((
        ClassEntryDelta {
          class_key,
          src_day,
          src_timeslot,
          dst_day,
          dst_timeslot,
        },
        entry_index,
      )))
      .ok_or(MoveOneClassRandomError::RandomChosenDestinationFull)
  }

  pub(crate) fn move_one_class_random<R: rand::Rng>(
    &mut self,
    rng: &mut R,
  ) -> Result<ClassEntryDelta, MoveOneClassRandomError> {
    let (delta, entry_index) = self.move_one_class_random_delta(rng)?;
    let entry = self.class_entries.get_mut(entry_index).unwrap();
    entry.day = delta.dst_day;
    entry.timeslot = delta.dst_timeslot;
    let _ = entry;
    let calendar = self.get_calendar_mut(delta.class_key);
    *calendar.get_mut(delta.src_day, delta.src_timeslot) -= 1;
    *calendar.get_mut(delta.dst_day, delta.dst_timeslot) += 1;
    Ok(delta)
  }

  pub(crate) fn move_one_class(
    &mut self,
    source_day_idx: week_calendar::Day,
    source_timeslot_idx: week_calendar::Timeslot,
    target_day_idx: week_calendar::Day,
    target_timeslot_idx: week_calendar::Timeslot,
    class_key: ClassKey,
  ) {
    self
      .remove_one_class(source_day_idx, source_timeslot_idx, class_key)
      .unwrap();
    self
      .add_one_class(target_day_idx, target_timeslot_idx, class_key)
      .unwrap();
  }

  pub(crate) fn new_class(&mut self) -> ClassKey {
    self.data.insert(Default::default())
  }

  /// Should only be used for testing. Otherwise call through SchoolSchedule
  pub(crate) fn add_one_class(
    &mut self,
    day: week_calendar::Day,
    timeslot: week_calendar::Timeslot,
    class_key: ClassKey,
  ) -> Result<u8, AddOneClassError> {
    let calendar = self.get_calendar_mut(class_key);
    let r = calendar.get(day, timeslot).checked_add(1);
    match r {
      Some(new_count) => {
        *calendar.get_mut(day, timeslot) = new_count;
        self.class_entries.push(SingleClassEntry {
          day,
          timeslot,
          class_key,
        });
        Ok(new_count)
      }
      None => Err(AddOneClassError::DestinationFull),
    }
  }

  /// Time complexity linear with the total count of classes in calendar
  pub(super) fn remove_one_class(
    &mut self,
    day: week_calendar::Day,
    timeslot: week_calendar::Timeslot,
    class_key: ClassKey,
  ) -> Result<u8, RemoveOneClassError> {
    let entry_idx = self
      .class_entries
      .iter()
      .position(|x| {
        *x == SingleClassEntry {
          day,
          timeslot,
          class_key,
        }
      })
      .ok_or(RemoveOneClassError::SourceEmpty)?;
    self.class_entries.swap_remove(entry_idx);
    let new_count = self
      .get_calendar_mut(class_key)
      .get_mut(day, timeslot)
      .checked_sub(1)
      .expect("If ClassEntry was present, the count in the calendar should be at least one.");
    *self.get_calendar_mut(class_key).get_mut(day, timeslot) = new_count;
    Ok(new_count)
  }

  /// Time complexity linear with the total count of classes in calendar
  pub(super) fn remove_one_class_anywhere(
    &mut self,
    class_key: ClassKey,
  ) -> Result<u8, RemoveOneClassAnywhereError> {
    let entry_idx = self
      .class_entries
      .iter()
      .position(|x| x.class_key == class_key)
      .ok_or(RemoveOneClassAnywhereError::NoClasses)?;
    let entry = self.class_entries.swap_remove(entry_idx);
    let new_count = self
      .get_calendar(class_key)
      .get(entry.day, entry.timeslot)
      .checked_sub(1)
      .expect("If ClassEntry was present, the count in the calendar should be at least one.");
    *self
      .get_calendar_mut(class_key)
      .get_mut(entry.day, entry.timeslot) = new_count;
    Ok(new_count)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn class_calendar_test() {
    let mut class_calendar = ClassCalendar::default();
    let k1 = class_calendar.new_class();
    let k2 = class_calendar.new_class();
    let _ = class_calendar.new_class();
    let k3 = class_calendar.new_class();
    let d0: week_calendar::Day = 0.try_into().unwrap();
    let t1: week_calendar::Timeslot = 1.try_into().unwrap();

    class_calendar.add_one_class(d0, t1, k3).unwrap();
    class_calendar.add_one_class(d0, t1, k3).unwrap();
    assert_eq!(class_calendar.get_count(d0, t1, k1), 0);
    assert_eq!(class_calendar.get_count(d0, t1, k2), 0);
    assert_eq!(class_calendar.get_count(d0, t1, k3), 2);
    class_calendar.remove_one_class_anywhere(k3).unwrap();
    assert_eq!(class_calendar.get_count(d0, t1, k3), 1);
    class_calendar.add_one_class(d0, t1, k3).unwrap();
    class_calendar.add_one_class(d0, t1, k3).unwrap();
    class_calendar.add_one_class(d0, t1, k3).unwrap();
    class_calendar.add_one_class(d0, t1, k3).unwrap();
    class_calendar.add_one_class(d0, t1, k3).unwrap();
    assert_eq!(class_calendar.get_count(d0, t1, k3), 6);
    class_calendar.remove_one_class_anywhere(k3).unwrap();
    class_calendar.remove_one_class_anywhere(k3).unwrap();
    class_calendar.remove_one_class_anywhere(k3).unwrap();
    class_calendar.remove_one_class_anywhere(k3).unwrap();
    assert_eq!(class_calendar.get_count(d0, t1, k3), 2);
    class_calendar.remove_one_class(d0, t1, k3).unwrap();
    assert_eq!(class_calendar.get_count(d0, t1, k3), 1);
    class_calendar.remove_one_class_anywhere(k3).unwrap();
    assert_eq!(class_calendar.get_count(d0, t1, k3), 0);
  }
}
