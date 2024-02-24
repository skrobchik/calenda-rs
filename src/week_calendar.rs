use serde::{Deserialize, Serialize};

use crate::timeslot;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct WeekCalendar<T> {
  data: Vec<T>,
}

impl<T: Default + Clone> Default for WeekCalendar<T> {
  fn default() -> Self {
    Self {
      data: vec![Default::default(); timeslot::TIMESLOT_COUNT * timeslot::DAY_COUNT],
    }
  }
}

fn get_index(day: timeslot::Day, timeslot: timeslot::Timeslot) -> usize {
  usize::from(day) * timeslot::TIMESLOT_COUNT + usize::from(timeslot)
}

impl<T> WeekCalendar<T> {
  pub(crate) fn get(&self, day: timeslot::Day, timeslot: timeslot::Timeslot) -> &T {
    &self.data[get_index(day, timeslot)]
  }

  pub(crate) fn get_mut(&mut self, day: timeslot::Day, timeslot: timeslot::Timeslot) -> &mut T {
    &mut self.data[get_index(day, timeslot)]
  }
}
