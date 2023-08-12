use std::ops::Range;

use rand::{prelude::StdRng, Rng};
use serde::{Deserialize, Serialize};

pub(crate) const TIMESLOT_COUNT: usize = 24;
pub(crate) const DAY_COUNT: usize = 5;

pub(crate) const TIMESLOT_RANGE: Range<usize> = 0..TIMESLOT_COUNT;
pub(crate) const DAY_RANGE: Range<usize> = 0..DAY_COUNT;

pub(crate) type Timeslot = usize;
pub(crate) type Day = usize;

#[derive(Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub(crate) struct DayTimeSlot {
  pub day: Day,
  pub timeslot: Timeslot,
}

impl DayTimeSlot {
  pub(crate) fn new(day: Day, timeslot: Timeslot) -> DayTimeSlot {
    DayTimeSlot { day, timeslot }
  }
  pub(crate) fn random(rng: &mut StdRng) -> DayTimeSlot {
    DayTimeSlot {
      day: rng.gen_range(DAY_RANGE),
      timeslot: rng.gen_range(TIMESLOT_RANGE),
    }
  }
}

#[allow(unused)]
pub(crate) const TIMESLOT_08_00: usize = 0;
#[allow(unused)]
pub(crate) const TIMESLOT_08_30: usize = 1;
#[allow(unused)]
pub(crate) const TIMESLOT_09_00: usize = 2;
#[allow(unused)]
pub(crate) const TIMESLOT_09_30: usize = 3;
#[allow(unused)]
pub(crate) const TIMESLOT_10_00: usize = 4;
#[allow(unused)]
pub(crate) const TIMESLOT_10_30: usize = 5;
#[allow(unused)]
pub(crate) const TIMESLOT_11_00: usize = 6;
#[allow(unused)]
pub(crate) const TIMESLOT_11_30: usize = 7;
#[allow(unused)]
pub(crate) const TIMESLOT_12_00: usize = 8;
#[allow(unused)]
pub(crate) const TIMESLOT_12_30: usize = 9;
#[allow(unused)]
pub(crate) const TIMESLOT_13_00: usize = 10;
#[allow(unused)]
pub(crate) const TIMESLOT_13_30: usize = 11;
#[allow(unused)]
pub(crate) const TIMESLOT_14_00: usize = 12;
#[allow(unused)]
pub(crate) const TIMESLOT_14_30: usize = 13;
#[allow(unused)]
pub(crate) const TIMESLOT_15_00: usize = 14;
#[allow(unused)]
pub(crate) const TIMESLOT_15_30: usize = 15;
#[allow(unused)]
pub(crate) const TIMESLOT_16_00: usize = 16;
#[allow(unused)]
pub(crate) const TIMESLOT_16_30: usize = 17;
#[allow(unused)]
pub(crate) const TIMESLOT_17_00: usize = 18;
#[allow(unused)]
pub(crate) const TIMESLOT_17_30: usize = 19;
#[allow(unused)]
pub(crate) const TIMESLOT_18_00: usize = 20;
#[allow(unused)]
pub(crate) const TIMESLOT_18_30: usize = 21;
#[allow(unused)]
pub(crate) const TIMESLOT_19_00: usize = 22;
#[allow(unused)]
pub(crate) const TIMESLOT_19_30: usize = 23;
