use serde::{Deserialize, Serialize};

use std::ops::Range;

pub(crate) const TIMESLOT_COUNT: usize = 12;
pub(crate) const DAY_COUNT: usize = 5;

const DAY_VALUE_RANGE: Range<usize> = std::ops::Range {
  start: 0,
  end: DAY_COUNT,
};

const TIMESLOT_VALUE_RANGE: Range<usize> = std::ops::Range {
  start: 0,
  end: TIMESLOT_COUNT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub(crate) struct Timeslot(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub(crate) struct Day(usize);

#[derive(thiserror::Error, Debug)]
#[error("Timeslot value is outside of range")]
pub(crate) struct TimeslotValueOutOfRangeError {}

impl TryFrom<usize> for Timeslot {
  type Error = TimeslotValueOutOfRangeError;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    if TIMESLOT_VALUE_RANGE.contains(&value) {
      Ok(Timeslot(value))
    } else {
      Err(TimeslotValueOutOfRangeError {})
    }
  }
}

impl From<Timeslot> for usize {
  fn from(val: Timeslot) -> Self {
    val.0
  }
}

#[derive(thiserror::Error, Debug)]
#[error("Day value is outside of range")]
pub(crate) struct DayValueOutOfRangeError {}

impl TryFrom<usize> for Day {
  type Error = DayValueOutOfRangeError;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    if DAY_VALUE_RANGE.contains(&value) {
      Ok(Day(value))
    } else {
      Err(DayValueOutOfRangeError {})
    }
  }
}

impl Timeslot {
  pub(crate) fn new_random<R: rand::Rng>(rng: &mut R) -> Self {
    Self(rng.gen_range(TIMESLOT_VALUE_RANGE))
  }

  pub(crate) fn all() -> impl ExactSizeIterator<Item = Self> {
    TIMESLOT_VALUE_RANGE.map(Timeslot)
  }
}

impl Day {
  pub(crate) fn new_random<R: rand::Rng>(rng: &mut R) -> Self {
    Self(rng.gen_range(DAY_VALUE_RANGE))
  }

  pub(crate) fn all() -> impl ExactSizeIterator<Item = Self> {
    DAY_VALUE_RANGE.map(Day)
  }
}

impl From<Day> for usize {
  fn from(val: Day) -> Self {
    val.0
  }
}

pub(crate) fn timeslot_to_hour(timeslot: Timeslot) -> u32 {
  (timeslot.0 as u32) + 8
}

#[allow(unused)]
pub(crate) const TIMESLOT_08_00: usize = 0;
#[allow(unused)]
pub(crate) const TIMESLOT_09_00: usize = 1;
#[allow(unused)]
pub(crate) const TIMESLOT_10_00: usize = 2;
#[allow(unused)]
pub(crate) const TIMESLOT_11_00: usize = 3;
#[allow(unused)]
pub(crate) const TIMESLOT_12_00: usize = 4;
#[allow(unused)]
pub(crate) const TIMESLOT_13_00: usize = 5;
#[allow(unused)]
pub(crate) const TIMESLOT_14_00: usize = 6;
#[allow(unused)]
pub(crate) const TIMESLOT_15_00: usize = 7;
#[allow(unused)]
pub(crate) const TIMESLOT_16_00: usize = 8;
#[allow(unused)]
pub(crate) const TIMESLOT_17_00: usize = 9;
#[allow(unused)]
pub(crate) const TIMESLOT_18_00: usize = 10;
#[allow(unused)]
pub(crate) const TIMESLOT_19_00: usize = 11;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct WeekCalendar<T> {
  data: Vec<T>,
}

impl<T: Default + Clone> Default for WeekCalendar<T> {
  fn default() -> Self {
    Self {
      data: vec![Default::default(); TIMESLOT_COUNT * DAY_COUNT],
    }
  }
}

fn get_index(day: Day, timeslot: Timeslot) -> usize {
  usize::from(day) * TIMESLOT_COUNT + usize::from(timeslot)
}

impl<T> WeekCalendar<T> {
  pub(crate) fn get(&self, day: Day, timeslot: Timeslot) -> &T {
    &self.data[get_index(day, timeslot)]
  }

  pub(crate) fn get_mut(&mut self, day: Day, timeslot: Timeslot) -> &mut T {
    &mut self.data[get_index(day, timeslot)]
  }
}
