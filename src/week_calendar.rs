use std::{ops::{Index, IndexMut}, marker::PhantomData, rc::Rc};

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::timeslot::*;

pub const DAY_COUNT: usize = 7; // 7 days in a week

#[deprecated]
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WeekCalendar<T> {
  data: Vec<T>,
}

impl<T: Default + Clone> Default for WeekCalendar<T> {
  fn default() -> Self {
    Self {
      data: std::iter::repeat(Default::default()).take(TIMESLOT_COUNT*DAY_COUNT).collect(),
    }
  }
}

impl<T> WeekCalendar<T> {
  pub fn get<'a, I1: Into<usize>, I2: Into<usize>>(&'a self, day: I1, timeslot: I2) -> Option<&'a T> {
    let day: usize = day.into();
    let timeslot: usize = timeslot.into();
    if !(0..DAY_COUNT).contains(&day) {
      return None;
    }
    if !(0..TIMESLOT_COUNT).contains(&timeslot) {
      return None;
    }
    Some(&self.data[day*TIMESLOT_COUNT+timeslot])
  }

  pub fn get_mut<'a, I1: Into<usize>, I2: Into<usize>>(&'a mut self, day: I1, timeslot: I2) -> Option<&'a mut T> {
    let day: usize = day.into();
    let timeslot: usize = timeslot.into();
    if !(0..DAY_COUNT).contains(&day) {
      return None;
    }
    if !(0..TIMESLOT_COUNT).contains(&timeslot) {
      return None;
    }
    Some(&mut self.data[day*TIMESLOT_COUNT+timeslot])
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
