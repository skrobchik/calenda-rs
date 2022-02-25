use rand::{
    prelude::{IteratorRandom, ThreadRng},
    Rng,
};

use crate::{
    calendars::CalendarState,
    timeslot::{DayTimeSlot, DAY_COUNT, TIMESLOT_COUNT},
};

pub trait Perturbator {
    fn perturbate(&self, state: &mut CalendarState, rng: &mut ThreadRng);
}

pub struct MoveHour;

impl MoveHour {
    pub fn new() -> Self {
        Self {}
    }
}

impl Perturbator for MoveHour {
    fn perturbate(&self, state: &mut CalendarState, rng: &mut ThreadRng) {
        let s = state.get_session_set().keys().choose(rng);
        if s.is_none() {
            return;
        }
        let session = s.unwrap();
        let source = session.t.clone();
        let target_timeslot = {
            if source.timeslot == 0 {
                source.timeslot + 1
            } else if source.timeslot == TIMESLOT_COUNT - 1 {
                source.timeslot - 1
            } else if rng.gen_ratio(1, 2) {
                source.timeslot + 1
            } else {
                source.timeslot - 1
            }
        };
        let target = DayTimeSlot {
            day: source.day,
            timeslot: target_timeslot,
        };
        let class_id = session.class_id;
        let r = state.move_session(class_id, source, target);
        if r.is_err() {
            ()
        }
    }
}

pub struct MoveDay;

impl MoveDay {
    pub fn new() -> Self {
        Self {}
    }
}

impl Perturbator for MoveDay {
    fn perturbate(&self, state: &mut CalendarState, rng: &mut ThreadRng) {
        let s = state.get_session_set().keys().choose(rng);
        if s.is_none() {
            return;
        }
        let session = s.unwrap();
        let source = session.t.clone();
        let target_day = {
            if source.day == 0 {
                source.day + 1
            } else if source.day == DAY_COUNT - 1 {
                source.day - 1
            } else if rng.gen_ratio(1, 2) {
                source.day + 1
            } else {
                source.day - 1
            }
        };
        let target = DayTimeSlot {
            day: target_day,
            timeslot: rng.gen_range(crate::timeslot::TIMESLOT_RANGE),
        };
        let class_id = session.class_id;
        let r = state.move_session(class_id, source, target);
        if r.is_err() {
            ()
        }
    }
}
