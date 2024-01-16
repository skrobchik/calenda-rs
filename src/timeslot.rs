use std::ops::Range;

pub(crate) const TIMESLOT_COUNT: usize = 12;
pub(crate) const DAY_COUNT: usize = 5;

pub(crate) const TIMESLOT_RANGE: Range<usize> = 0..TIMESLOT_COUNT;
pub(crate) const DAY_RANGE: Range<usize> = 0..DAY_COUNT;

pub(crate) type Timeslot = usize;
pub(crate) type Day = usize;

pub(crate) fn timeslot_to_hour(timeslot: Timeslot) -> u32 {
  (timeslot as u32) + 8
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
