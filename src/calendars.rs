use rand::prelude::IteratorRandom;
use rand::prelude::StdRng;
use rand::Rng;

use crate::real_counter::RealCounter;
use crate::timeslot::DayTimeSlot;
use crate::timeslot::DAY_COUNT;
use crate::timeslot::DAY_RANGE;
use crate::timeslot::TIMESLOT_COUNT;
use crate::timeslot::TIMESLOT_RANGE;
use std::{collections::HashMap, fmt::Display};

/// Returns x-1 or x+1 randomly as long as the result is withing the range's bounds. If x-1 or x+1 are not inside the range, returns x.
fn random_shift_bounded(x: usize, range: std::ops::Range<usize>, rng: &mut StdRng) -> usize {
    let smaller_available = range.start < x;
    let larger_available = range.end > x + 1;
    if !smaller_available && !larger_available {
        return x;
    }
    if !smaller_available && larger_available {
        return x + 1;
    }
    if smaller_available && !larger_available {
        return x - 1;
    }
    match rng.gen_bool(0.5) {
        true => x - 1,
        false => x + 1,
    }
}

pub type ClassId = usize;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Session {
    pub class_id: ClassId,
    pub t: DayTimeSlot,
}

#[derive(Clone)]
pub struct CalendarState {
    class_schedules: HashMap<ClassId, [[usize; TIMESLOT_COUNT]; DAY_COUNT]>,
    schedule_matrix: [[RealCounter<ClassId>; TIMESLOT_COUNT]; DAY_COUNT],
    session_set: RealCounter<Session>,
}

impl Default for CalendarState {
    fn default() -> Self {
        Self {
            class_schedules: Default::default(),
            schedule_matrix: Default::default(),
            session_set: Default::default(),
        }
    }
}

impl CalendarState {
    pub fn get_class_schedules(&self) -> &HashMap<ClassId, [[usize; TIMESLOT_COUNT]; DAY_COUNT]> {
        &self.class_schedules
    }

    pub fn get_session_set(&self) -> &RealCounter<Session> {
        &self.session_set
    }

    pub fn get_schedule_matrix(&self) -> &[[RealCounter<ClassId>; TIMESLOT_COUNT]; DAY_COUNT] {
        &self.schedule_matrix
    }

    pub fn move_session(
        &mut self,
        class_id: usize,
        source: DayTimeSlot,
        target: DayTimeSlot,
    ) -> Result<(), ()> {
        let source_sessions = &mut self.schedule_matrix[source.day][source.timeslot];
        source_sessions.decrement(&class_id).ok_or(())?;
        let target_sessions = &mut self.schedule_matrix[target.day][target.timeslot];
        target_sessions.increment(class_id);

        let class_schedule = self.class_schedules.get_mut(&class_id).ok_or(())?;
        class_schedule[source.day][source.timeslot] -= 1;
        class_schedule[target.day][target.timeslot] += 1;

        self.session_set
            .decrement(&Session {
                class_id,
                t: source,
            })
            .ok_or(())?;
        self.session_set.increment(Session {
            class_id,
            t: target,
        });

        Ok(())
    }

    pub fn get_random_neighbor(&self, rng: &mut StdRng) -> Option<CalendarState> {
        let session = self.get_session_set().keys().choose(rng)?;
        let source_daytime = session.t.clone();
        let target_daytime = match rng.gen_range(0u8..=1) {
            0 => DayTimeSlot {
                day: source_daytime.day,
                timeslot: random_shift_bounded(source_daytime.timeslot, TIMESLOT_RANGE, rng),
            },
            1 => DayTimeSlot {
                day: random_shift_bounded(source_daytime.day, DAY_RANGE, rng),
                timeslot: TIMESLOT_RANGE.choose(rng).unwrap(),
            },
            _ => unreachable!(),
        };
        let mut neighbor = self.clone();
        neighbor
            .move_session(session.class_id, source_daytime, target_daytime)
            .expect("Something went horribly wrong");
        Some(neighbor)
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_session(&mut self, class_id: usize, t: DayTimeSlot) {
        self.schedule_matrix[t.day][t.timeslot].increment(class_id);
        self.class_schedules.entry(class_id).or_default()[t.day][t.timeslot] += 1;
        self.session_set.increment(Session { class_id, t });
    }

    pub fn remove_class(&mut self, class_id: usize) {
        for t in TIMESLOT_RANGE {
            for d in DAY_RANGE {
                let courses = &mut self.schedule_matrix[d][t];
                courses.remove(&class_id);
            }
        }
        self.class_schedules.remove(&class_id);
        self.session_set = self
            .session_set
            .iter()
            .filter(|(s, _v)| s.class_id != class_id)
            .map(|(s, v)| (s.to_owned(), v.to_owned()))
            .collect();
    }

    pub fn clear(&mut self) {
        for t in TIMESLOT_RANGE {
            for d in DAY_RANGE {
                let courses = &mut self.schedule_matrix[d][t];
                courses.clear();
            }
        }
        self.class_schedules.clear();
        self.session_set.clear();
    }
}

impl Display for CalendarState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_len = self
            .schedule_matrix
            .iter()
            .flatten()
            .max_by(|x, y| x.count_total().cmp(&y.count_total()));
        if max_len.is_none() {
            return write!(f, "Empty Calendar");
        }
        let width = 1.max(max_len.unwrap().count_total());
        for t in TIMESLOT_RANGE {
            for d in DAY_RANGE {
                let courses = &self.schedule_matrix[d][t];
                write!(
                    f,
                    "{}{} ",
                    courses
                        .iter()
                        .map(
                            |(course_id, count)| std::iter::repeat(course_id.to_string())
                                .take(*count)
                                .collect::<String>()
                        )
                        .collect::<String>(),
                    std::iter::repeat('-')
                        .take(width - courses.count_total())
                        .collect::<String>()
                )?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
