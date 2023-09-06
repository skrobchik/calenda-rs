use serde::{Deserialize, Serialize};

use crate::timeslot::{DAY_COUNT, TIMESLOT_COUNT};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct WeekCalendar<T> {
  pub(crate) data: Vec<T>,
}

impl<T: Default + Clone> Default for WeekCalendar<T> {
  fn default() -> Self {
    Self {
      data: std::iter::repeat(Default::default())
        .take(TIMESLOT_COUNT * DAY_COUNT)
        .collect(),
    }
  }
}

impl<T> WeekCalendar<T> {
  pub(crate) fn get<I1: Into<usize>, I2: Into<usize>>(&self, day: I1, timeslot: I2) -> Option<&T> {
    let day: usize = day.into();
    let timeslot: usize = timeslot.into();
    if !(0..DAY_COUNT).contains(&day) {
      return None;
    }
    if !(0..TIMESLOT_COUNT).contains(&timeslot) {
      return None;
    }
    Some(&self.data[day * TIMESLOT_COUNT + timeslot])
  }

  pub(crate) fn get_mut<I1: Into<usize>, I2: Into<usize>>(
    &mut self,
    day: I1,
    timeslot: I2,
  ) -> Option<&mut T> {
    let day: usize = day.into();
    let timeslot: usize = timeslot.into();
    if !(0..DAY_COUNT).contains(&day) {
      return None;
    }
    if !(0..TIMESLOT_COUNT).contains(&timeslot) {
      return None;
    }
    Some(&mut self.data[day * TIMESLOT_COUNT + timeslot])
  }
}
