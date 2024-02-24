use std::ops::Range;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

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

impl TryFrom<usize> for Timeslot {
  type Error = anyhow::Error;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    if TIMESLOT_VALUE_RANGE.contains(&value) {
      Ok(Timeslot(value))
    } else {
      Err(anyhow!(
        "value `{:?}` outside timeslot range `{:?}`",
        value,
        TIMESLOT_VALUE_RANGE
      ))
    }
  }
}

impl From<Timeslot> for usize {
  fn from(val: Timeslot) -> Self {
    val.0
  }
}

impl TryFrom<usize> for Day {
  type Error = anyhow::Error;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    if DAY_VALUE_RANGE.contains(&value) {
      Ok(Day(value))
    } else {
      Err(anyhow!(
        "value `{:?}` outside day range `{:?}`",
        value,
        DAY_VALUE_RANGE
      ))
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
