use crate::real_counter::RealCounter;
use crate::timeslot::DayTimeSlot;
use crate::timeslot::DAY_COUNT;
use crate::timeslot::DAY_RANGE;
use crate::timeslot::TIMESLOT_COUNT;
use crate::timeslot::TIMESLOT_RANGE;
use std::{collections::HashMap, fmt::Display};

pub type ClassId = usize;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Session {
    pub class_id: ClassId,
    pub t: DayTimeSlot,
}

#[derive(Clone)]
pub struct CalendarState {
    class_schedules: HashMap<ClassId, RealCounter<DayTimeSlot>>,
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
    pub fn get_class_schedules(&self) -> &HashMap<ClassId, RealCounter<DayTimeSlot>> {
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
        class_schedule.decrement(&source).ok_or(())?;
        class_schedule.increment(target.clone());

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

    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_session(&mut self, class_id: usize, t: DayTimeSlot) {
        self.schedule_matrix[t.day][t.timeslot].increment(class_id);
        self.class_schedules
            .entry(class_id)
            .or_default()
            .increment(t.clone());
        self.session_set.increment(Session { class_id, t });
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
