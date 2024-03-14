use itertools::Itertools;

use crate::week_calendar;
use crate::week_calendar::WeekCalendar;
use serde::Deserialize;
use serde::Serialize;

pub const NUM_CLASS_IDS: usize = 256;
const CLASS_ID_VALUE_RANGE: std::ops::Range<usize> = 0..NUM_CLASS_IDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClassId(usize);

#[derive(thiserror::Error, Debug)]
#[error("Class id value is greater than maximum")]
pub struct ClassIdOutOfRangeError {}

impl ClassId {
  pub fn all() -> impl ExactSizeIterator<Item = Self> {
    CLASS_ID_VALUE_RANGE.map(ClassId)
  }
}

impl TryFrom<usize> for ClassId {
  type Error = ClassIdOutOfRangeError;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    if CLASS_ID_VALUE_RANGE.contains(&value) {
      Ok(Self(value))
    } else {
      Err(Self::Error {})
    }
  }
}

impl From<ClassId> for usize {
  fn from(value: ClassId) -> Self {
    value.0
  }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct SingleClassEntry {
  pub day_idx: week_calendar::Day,
  pub timeslot_idx: week_calendar::Timeslot,
  pub class_id: ClassId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassEntryDelta {
  pub class_id: ClassId,
  pub src_day_idx: week_calendar::Day,
  pub src_timeslot_idx: week_calendar::Timeslot,
  pub dst_day_idx: week_calendar::Day,
  pub dst_timeslot_idx: week_calendar::Timeslot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassCalendar {
  data: WeekCalendar<Vec<u8>>,
  class_entries: Vec<SingleClassEntry>,
}

impl Default for ClassCalendar {
  fn default() -> Self {
    Self {
      data: WeekCalendar::try_from(vec![
        vec![0_u8; NUM_CLASS_IDS];
        week_calendar::DAY_COUNT * week_calendar::TIMESLOT_COUNT
      ])
      .unwrap(),
      class_entries: Default::default(),
    }
  }
}

impl ClassCalendar {
  pub fn iter_timeslots(&self) -> impl Iterator<Item = &Vec<u8>> {
    week_calendar::Day::all()
      .flat_map(move |d| week_calendar::Timeslot::all().map(move |t| self.data.get(d, t)))
  }

  pub fn get_entries(&self) -> &Vec<SingleClassEntry> {
    &self.class_entries
  }

  pub fn get_timeslot(
    &self,
    day: week_calendar::Day,
    timeslot: week_calendar::Timeslot,
  ) -> &Vec<u8> {
    self.data.get(day, timeslot)
  }

  pub fn get_count(
    &self,
    day: week_calendar::Day,
    timeslot: week_calendar::Timeslot,
    class_id: ClassId,
  ) -> u8 {
    *self
      .data
      .get(day, timeslot)
      .get(usize::from(class_id))
      .unwrap()
  }

  pub fn move_one_class_random<R: rand::Rng>(&mut self, rng: &mut R) -> ClassEntryDelta {
    let entry_idx = rng.gen_range(0..self.class_entries.len());
    let entry = self.class_entries.get_mut(entry_idx).unwrap();
    let class_id = entry.class_id;
    let src_day = entry.day_idx;
    let src_timeslot = entry.timeslot_idx;
    let dst_day = week_calendar::Day::new_random(rng);
    let dst_timeslot = week_calendar::Timeslot::new_random(rng);
    entry.day_idx = dst_day;
    entry.timeslot_idx = dst_timeslot;
    {
      let src_timeslot = self.data.get_mut(src_day, src_timeslot);
      assert!(src_timeslot[usize::from(class_id)] >= 1);
      src_timeslot[usize::from(class_id)] -= 1;
    }
    {
      let dst_timeslot = self.data.get_mut(dst_day, dst_timeslot);
      dst_timeslot[usize::from(class_id)] =
        dst_timeslot[usize::from(class_id)].checked_add(1).unwrap();
    }
    ClassEntryDelta {
      class_id,
      src_day_idx: src_day,
      src_timeslot_idx: src_timeslot,
      dst_day_idx: dst_day,
      dst_timeslot_idx: dst_timeslot,
    }
  }

  pub fn move_one_class(
    &mut self,
    source_day_idx: week_calendar::Day,
    source_timeslot_idx: week_calendar::Timeslot,
    target_day_idx: week_calendar::Day,
    target_timeslot_idx: week_calendar::Timeslot,
    class_id: ClassId,
  ) {
    self.remove_one_class(source_day_idx, source_timeslot_idx, class_id);
    self.add_one_class(target_day_idx, target_timeslot_idx, class_id);
  }

  pub fn add_one_class(
    &mut self,
    day_idx: week_calendar::Day,
    timeslot_idx: week_calendar::Timeslot,
    class_id: ClassId,
  ) {
    let timeslot = self.data.get_mut(day_idx, timeslot_idx);
    timeslot[usize::from(class_id)] = timeslot[usize::from(class_id)].checked_add(1).unwrap();
    self.class_entries.push(SingleClassEntry {
      day_idx,
      timeslot_idx,
      class_id,
    });
  }

  pub fn remove_one_class(
    &mut self,
    day: week_calendar::Day,
    timeslot_idx: week_calendar::Timeslot,
    class_id: ClassId,
  ) {
    let timeslot = self.data.get_mut(day, timeslot_idx);
    timeslot[usize::from(class_id)] = timeslot[usize::from(class_id)].checked_sub(1).unwrap();

    let entry_idx = self
      .class_entries
      .iter()
      .find_position(|x| {
        **x
          == SingleClassEntry {
            day_idx: day,
            timeslot_idx,
            class_id,
          }
      })
      .unwrap()
      .0;
    self.class_entries.swap_remove(entry_idx);
  }

  pub(super) fn remove_one_class_anywhere(&mut self, class_id: ClassId) {
    let (entry_idx, entry) = self
      .class_entries
      .iter()
      .find_position(|e| e.class_id == class_id)
      .unwrap();

    let timeslot_idx = entry.timeslot_idx;
    let day_idx = entry.day_idx;

    let timeslot = self.data.get_mut(day_idx, timeslot_idx);

    timeslot[usize::from(class_id)] = timeslot[usize::from(class_id)].checked_sub(1).unwrap();

    self.class_entries.swap_remove(entry_idx);
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn class_calendar_test() {
    let mut class_calendar = ClassCalendar::default();
    class_calendar.add_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    class_calendar.add_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        3.try_into().unwrap()
      ),
      0
    );
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        4.try_into().unwrap()
      ),
      2
    );
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        5.try_into().unwrap()
      ),
      0
    );
    class_calendar.remove_one_class_anywhere(4.try_into().unwrap());
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        4.try_into().unwrap()
      ),
      1
    );
    class_calendar.add_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    class_calendar.add_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    class_calendar.add_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    class_calendar.add_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    class_calendar.add_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        4.try_into().unwrap()
      ),
      6
    );
    class_calendar.remove_one_class_anywhere(4.try_into().unwrap());
    class_calendar.remove_one_class_anywhere(4.try_into().unwrap());
    class_calendar.remove_one_class_anywhere(4.try_into().unwrap());
    class_calendar.remove_one_class_anywhere(4.try_into().unwrap());
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        4.try_into().unwrap()
      ),
      2
    );
    class_calendar.remove_one_class(
      0.try_into().unwrap(),
      1.try_into().unwrap(),
      4.try_into().unwrap(),
    );
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        4.try_into().unwrap()
      ),
      1
    );
    class_calendar.remove_one_class_anywhere(4.try_into().unwrap());
    assert_eq!(
      class_calendar.get_count(
        0.try_into().unwrap(),
        1.try_into().unwrap(),
        4.try_into().unwrap()
      ),
      0
    );
  }

  #[test]
  fn classes_iter_size_test() {
    let class_calendar = ClassCalendar::default();
    assert_eq!(
      class_calendar.iter_timeslots().count(),
      week_calendar::DAY_COUNT * week_calendar::TIMESLOT_COUNT
    );
  }
}
