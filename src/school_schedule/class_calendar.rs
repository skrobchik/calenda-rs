use itertools::Itertools;

use crate::week_calendar;
use crate::week_calendar::WeekCalendar;
use serde::Deserialize;
use serde::Serialize;

pub const MAX_CLASS_ID: usize = 256;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
struct SingleClassEntry {
  day_idx: week_calendar::Day,
  timeslot_idx: week_calendar::Timeslot,
  class_id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClassEntryDelta {
  pub(crate) class_id: usize,
  pub(crate) src_day_idx: week_calendar::Day,
  pub(crate) src_timeslot_idx: week_calendar::Timeslot,
  pub(crate) dst_day_idx: week_calendar::Day,
  pub(crate) dst_timeslot_idx: week_calendar::Timeslot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ClassCalendar {
  data: WeekCalendar<Vec<u8>>,
  class_entries: Vec<SingleClassEntry>,
}

impl ClassCalendar {
  pub(crate) fn iter_timeslots(&self) -> impl Iterator<Item = &Vec<u8>> {
    week_calendar::Day::all()
      .flat_map(move |d| week_calendar::Timeslot::all().map(move |t| self.data.get(d, t)))
  }

  pub(crate) fn get_timeslot(
    &self,
    day: week_calendar::Day,
    timeslot: week_calendar::Timeslot,
  ) -> &Vec<u8> {
    self.data.get(day, timeslot)
  }

  pub(crate) fn get_count(
    &self,
    day: week_calendar::Day,
    timeslot: week_calendar::Timeslot,
    class_id: usize,
  ) -> u8 {
    assert!(class_id <= MAX_CLASS_ID);
    let timeslot = self.data.get(day, timeslot);
    timeslot.get(class_id).copied().unwrap_or(0_u8)
  }

  pub(crate) fn move_one_class_random<R: rand::Rng>(&mut self, rng: &mut R) -> ClassEntryDelta {
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
      assert!(src_timeslot[class_id] >= 1);
      src_timeslot[class_id] -= 1;
    }
    {
      let dst_timeslot = self.data.get_mut(dst_day, dst_timeslot);
      if class_id >= dst_timeslot.len() {
        dst_timeslot.resize(class_id + 1, 0);
      }
      dst_timeslot[class_id] = dst_timeslot[class_id].checked_add(1).unwrap();
    }
    ClassEntryDelta {
      class_id,
      src_day_idx: src_day,
      src_timeslot_idx: src_timeslot,
      dst_day_idx: dst_day,
      dst_timeslot_idx: dst_timeslot,
    }
  }

  pub(crate) fn move_one_class(
    &mut self,
    source_day_idx: week_calendar::Day,
    source_timeslot_idx: week_calendar::Timeslot,
    target_day_idx: week_calendar::Day,
    target_timeslot_idx: week_calendar::Timeslot,
    class_id: usize,
  ) {
    self.remove_one_class(source_day_idx, source_timeslot_idx, class_id);
    self.add_one_class(target_day_idx, target_timeslot_idx, class_id);
  }

  pub(crate) fn add_one_class(
    &mut self,
    day_idx: week_calendar::Day,
    timeslot_idx: week_calendar::Timeslot,
    class_id: usize,
  ) {
    assert!(class_id <= MAX_CLASS_ID);
    let timeslot = self.data.get_mut(day_idx, timeslot_idx);
    if class_id >= timeslot.len() {
      timeslot.resize(class_id + 1, 0_u8);
    }
    timeslot[class_id] = timeslot[class_id].checked_add(1).unwrap();
    self.class_entries.push(SingleClassEntry {
      day_idx,
      timeslot_idx,
      class_id,
    });
  }

  pub(crate) fn remove_one_class(
    &mut self,
    day: week_calendar::Day,
    timeslot_idx: week_calendar::Timeslot,
    class_id: usize,
  ) {
    assert!(class_id <= MAX_CLASS_ID);

    let timeslot = self.data.get_mut(day, timeslot_idx);
    assert!(timeslot.len() > class_id);

    timeslot[class_id] = timeslot[class_id].checked_sub(1).unwrap();

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

  pub(super) fn remove_one_class_anywhere(&mut self, class_id: usize) {
    let (entry_idx, entry) = self
      .class_entries
      .iter()
      .find_position(|e| e.class_id == class_id)
      .unwrap();

    let timeslot_idx = entry.timeslot_idx;
    let day_idx = entry.day_idx;

    let timeslot = self.data.get_mut(day_idx, timeslot_idx);

    timeslot[class_id] = timeslot[class_id].checked_sub(1).unwrap();

    self.class_entries.swap_remove(entry_idx);
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn class_calendar_test() {
    let mut class_calendar = ClassCalendar::default();
    class_calendar.add_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    class_calendar.add_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 3),
      0
    );
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 4),
      2
    );
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 5),
      0
    );
    class_calendar.remove_one_class_anywhere(4);
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 4),
      1
    );
    class_calendar.add_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    class_calendar.add_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    class_calendar.add_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    class_calendar.add_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    class_calendar.add_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 4),
      6
    );
    class_calendar.remove_one_class_anywhere(4);
    class_calendar.remove_one_class_anywhere(4);
    class_calendar.remove_one_class_anywhere(4);
    class_calendar.remove_one_class_anywhere(4);
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 4),
      2
    );
    class_calendar.remove_one_class(0.try_into().unwrap(), 1.try_into().unwrap(), 4);
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 4),
      1
    );
    class_calendar.remove_one_class_anywhere(4);
    assert_eq!(
      class_calendar.get_count(0.try_into().unwrap(), 1.try_into().unwrap(), 4),
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
