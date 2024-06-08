use serde::{Deserialize, Serialize};
use std::ops::Range;

const TIMESLOT_COUNT: usize = 12;
const DAY_COUNT: usize = 5;
const DATA_LEN: usize = TIMESLOT_COUNT * DAY_COUNT;

const DAY_VALUE_RANGE: Range<usize> = std::ops::Range {
  start: 0,
  end: DAY_COUNT,
};

const TIMESLOT_VALUE_RANGE: Range<usize> = std::ops::Range {
  start: 0,
  end: TIMESLOT_COUNT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timeslot(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Day(usize);

impl From<Timeslot> for usize {
  fn from(val: Timeslot) -> Self {
    val.0
  }
}

/// Contains is still not const, so we make our own for now
/// https://github.com/rust-lang/rust/issues/108082
/// This can be removed when the above feature is stabilised
const fn range_contains(range: Range<usize>, item: usize) -> bool {
  range.start <= item && item < range.end
}

impl Timeslot {
  pub fn new_random<R: rand::Rng>(rng: &mut R) -> Self {
    Self(rng.gen_range(TIMESLOT_VALUE_RANGE))
  }

  pub fn all() -> impl ExactSizeIterator<Item = Self> {
    TIMESLOT_VALUE_RANGE.map(Timeslot)
  }

  pub const fn from_usize(value: usize) -> Option<Self> {
    if range_contains(TIMESLOT_VALUE_RANGE, value) {
      Some(Timeslot(value))
    } else {
      None
    }
  }
}

impl Day {
  pub fn new_random<R: rand::Rng>(rng: &mut R) -> Self {
    Self(rng.gen_range(DAY_VALUE_RANGE))
  }

  pub fn all() -> impl ExactSizeIterator<Item = Self> {
    DAY_VALUE_RANGE.map(Day)
  }

  pub const fn from_usize(value: usize) -> Option<Self> {
    if range_contains(DAY_VALUE_RANGE, value) {
      Some(Day(value))
    } else {
      None
    }
  }
}

impl From<Day> for usize {
  fn from(val: Day) -> Self {
    val.0
  }
}

/// const Option::unwrap is not yet stable
const fn const_unwrap_timeslot(opt: Option<Timeslot>) -> Timeslot {
  match opt {
    Some(t) => t,
    None => panic!(),
  }
}
pub const TIMESLOT_08_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(0));
pub const TIMESLOT_09_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(1));
pub const TIMESLOT_10_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(2));
pub const TIMESLOT_11_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(3));
pub const TIMESLOT_12_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(4));
pub const TIMESLOT_13_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(5));
pub const TIMESLOT_14_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(6));
pub const TIMESLOT_15_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(7));
pub const TIMESLOT_16_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(8));
pub const TIMESLOT_17_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(9));
pub const TIMESLOT_18_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(10));
pub const TIMESLOT_19_00: Timeslot = const_unwrap_timeslot(Timeslot::from_usize(11));
const _: () = assert!(TIMESLOT_19_00.0 == TIMESLOT_COUNT - 1);

/// const Option::unwrap is not yet stable
const fn const_unwrap_day(opt: Option<Day>) -> Day {
  match opt {
    Some(d) => d,
    None => panic!(),
  }
}
pub const DAY_MONDAY: Day = const_unwrap_day(Day::from_usize(0));
pub const DAY_TUESDAY: Day = const_unwrap_day(Day::from_usize(1));
pub const DAY_WEDNESDAY: Day = const_unwrap_day(Day::from_usize(2));
pub const DAY_THURSDAY: Day = const_unwrap_day(Day::from_usize(3));
pub const DAY_FRIDAY: Day = const_unwrap_day(Day::from_usize(4));
const _: () = assert!(DAY_FRIDAY.0 == DAY_COUNT - 1);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WeekCalendar<T> {
  data: Vec<T>,
}

impl<T: Default + Clone> Default for WeekCalendar<T> {
  fn default() -> Self {
    Self {
      data: vec![Default::default(); DATA_LEN],
    }
  }
}

const fn get_index(day: Day, timeslot: Timeslot) -> usize {
  day.0 * TIMESLOT_COUNT + timeslot.0
}

impl<T> WeekCalendar<T> {
  pub fn get(&self, day: Day, timeslot: Timeslot) -> &T {
    &self.data[get_index(day, timeslot)]
  }

  pub fn get_mut(&mut self, day: Day, timeslot: Timeslot) -> &mut T {
    &mut self.data[get_index(day, timeslot)]
  }
}

#[derive(thiserror::Error, Debug)]
#[error("Provided data length is incorrect")]
pub struct IncorrectDataLenError {}
impl<T> TryFrom<Vec<T>> for WeekCalendar<T> {
  type Error = IncorrectDataLenError;

  fn try_from(data: Vec<T>) -> Result<Self, Self::Error> {
    if data.len() == DATA_LEN {
      Ok(Self { data })
    } else {
      Err(IncorrectDataLenError {})
    }
  }
}
