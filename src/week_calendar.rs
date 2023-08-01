use serde::{Deserialize, Serialize};

use crate::timeslot::*;

pub const DAY_COUNT: usize = 7; // 7 days in a week

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WeekCalendar<T> {
  pub data: Vec<T>,
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
  pub fn get<I1: Into<usize>, I2: Into<usize>>(
    &self,
    day: I1,
    timeslot: I2,
  ) -> Option<&T> {
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

  pub fn get_mut<I1: Into<usize>, I2: Into<usize>>(
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

#[derive(Clone, Copy)]
pub enum Weekday {
  Monday,
  Tuesday,
  Wednesday,
  Thursday,
  Friday,
  Saturday,
  Sunday,
}

impl From<Weekday> for usize {
  fn from(val: Weekday) -> Self {
    match val {
      Weekday::Monday => 0,
      Weekday::Tuesday => 1,
      Weekday::Wednesday => 2,
      Weekday::Thursday => 3,
      Weekday::Friday => 4,
      Weekday::Saturday => 5,
      Weekday::Sunday => 6,
    }
  }
}

impl TryFrom<usize> for Weekday {
  type Error = ();

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Weekday::Monday),
      1 => Ok(Weekday::Tuesday),
      2 => Ok(Weekday::Wednesday),
      3 => Ok(Weekday::Thursday),
      4 => Ok(Weekday::Friday),
      5 => Ok(Weekday::Saturday),
      6 => Ok(Weekday::Sunday),
      _ => Err(()),
    }
  }
}
