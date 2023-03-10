use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::timeslots::*;

pub const DAY_COUNT: usize = 7; // 7 days in a week

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DaySchedule<T> {
  #[serde(
    bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"),
    with = "BigArray"
  )]
  data: [T; TIMESLOT_COUNT],
}

impl<T: Default + Copy> Default for DaySchedule<T> {
  fn default() -> Self {
    let init: T = Default::default();
    Self {
      data: [init; TIMESLOT_COUNT],
    }
  }
}

impl<T> Index<usize> for DaySchedule<T> {
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    &self.data[index]
  }
}

impl<T> IndexMut<usize> for DaySchedule<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.data[index]
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WeekCalendar<T> {
  // Box because doesn't fit in stack
  data: Box<[DaySchedule<T>; DAY_COUNT]>,
}

impl<T> Index<usize> for WeekCalendar<T> {
  type Output = DaySchedule<T>;

  fn index(&self, index: usize) -> &Self::Output {
    &self.data[index]
  }
}

impl<T> IndexMut<usize> for WeekCalendar<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.data[index]
  }
}

impl<T: Default + Copy> Default for WeekCalendar<T> {
  fn default() -> Self {
    Self {
      data: Default::default(),
    }
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

impl Into<usize> for Weekday {
  fn into(self) -> usize {
    match self {
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
