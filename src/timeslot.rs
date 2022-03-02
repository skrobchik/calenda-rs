use std::ops::Range;

use rand::{prelude::StdRng, Rng};

pub const TIMESLOT_COUNT: usize = 24;
pub const DAY_COUNT: usize = 5;

pub const TIMESLOT_RANGE: Range<usize> = 0..TIMESLOT_COUNT;
pub const DAY_RANGE: Range<usize> = 0..DAY_COUNT;

pub type Timeslot = usize;
pub type Day = usize;

#[derive(Ord, PartialOrd, PartialEq, Eq, Hash, Clone)]
pub struct DayTimeSlot {
    pub day: Day,
    pub timeslot: Timeslot,
}

impl DayTimeSlot {
    pub fn new(day: Day, timeslot: Timeslot) -> DayTimeSlot {
        DayTimeSlot {
            day,
            timeslot
        }
    }
    pub fn random(rng: &mut StdRng) -> DayTimeSlot {
        DayTimeSlot {
            day: rng.gen_range(DAY_RANGE),
            timeslot: rng.gen_range(TIMESLOT_RANGE)
        }
    }
}

pub const TIMESLOT_08_00: usize = 0;
pub const TIMESLOT_08_30: usize = 1;
pub const TIMESLOT_09_00: usize = 2;
pub const TIMESLOT_09_30: usize = 3;
pub const TIMESLOT_10_00: usize = 4;
pub const TIMESLOT_10_30: usize = 5;
pub const TIMESLOT_11_00: usize = 6;
pub const TIMESLOT_11_30: usize = 7;
pub const TIMESLOT_12_00: usize = 8;
pub const TIMESLOT_12_30: usize = 9;
pub const TIMESLOT_13_00: usize = 10;
pub const TIMESLOT_13_30: usize = 11;
pub const TIMESLOT_14_00: usize = 12;
pub const TIMESLOT_14_30: usize = 13;
pub const TIMESLOT_15_00: usize = 14;
pub const TIMESLOT_15_30: usize = 15;
pub const TIMESLOT_16_00: usize = 16;
pub const TIMESLOT_16_30: usize = 17;
pub const TIMESLOT_17_00: usize = 18;
pub const TIMESLOT_17_30: usize = 19;
pub const TIMESLOT_18_00: usize = 20;
pub const TIMESLOT_18_30: usize = 21;
pub const TIMESLOT_19_00: usize = 22;
pub const TIMESLOT_19_30: usize = 23;