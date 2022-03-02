use crate::calendars::CalendarState;
use itertools::Itertools;

pub struct GapCount {
    weight: f32,
}

impl GapCount {
    pub fn new(weight: f32) -> Self {
        GapCount { weight }
    }
}

pub trait Evaluator {
    fn evaluate(&self, state: &CalendarState) -> f32;
}

impl Evaluator for GapCount {
    fn evaluate(&self, state: &CalendarState) -> f32 {
        let mut count = 0;
        for day in state.get_schedule_matrix().iter() {
            if let Some(last_class) = day.iter().enumerate().rev().find(|x| !x.1.is_empty()) {
                count += day
                    .iter()
                    .enumerate()
                    .skip_while(|(_i, x)| x.is_empty())
                    .skip(1)
                    .take_while(|(i, _x)| *i < last_class.0)
                    .filter(|(_i, x)| x.is_empty())
                    .count()
            }
        }
        count as f32 * self.weight
    }
}

pub struct Daylight {
    wake_up_time: usize,
    sleep_time: usize,
    weight: f32,
}

impl Daylight {
    pub fn new(wake_up_time: usize, sleep_time: usize, weight: f32) -> Self {
        Daylight {
            wake_up_time,
            sleep_time,
            weight,
        }
    }
}

impl Evaluator for Daylight {
    fn evaluate(&self, state: &CalendarState) -> f32 {
        state
            .get_session_set()
            .iter()
            .filter(|(session, _count)| {
                session.t.timeslot < self.wake_up_time || session.t.timeslot >= self.sleep_time
            })
            .map(|(_session, count)| count)
            .map(|x| x)
            .sum::<usize>() as f32
            * self.weight
    }
}

pub struct Colliding {
    weight: f32,
}

impl Evaluator for Colliding {
    fn evaluate(&self, state: &CalendarState) -> f32 {
        state
            .get_schedule_matrix()
            .iter()
            .flatten()
            .map(|x| x.count_total())
            .filter(|x| *x >= 2)
            .map(|x| x)
            .sum::<usize>() as f32
            * self.weight
    }
}

impl Colliding {
    pub fn new(weight: f32) -> Self {
        Colliding { weight }
    }
}

pub struct DailyWorkDifference {
    weight: f32,
}

impl DailyWorkDifference {
    pub fn new(weight: f32) -> Self {
        DailyWorkDifference { weight }
    }
}

impl Evaluator for DailyWorkDifference {
    fn evaluate(&self, state: &CalendarState) -> f32 {
        let mut max_sessions = 0;
        let mut min_sessions = usize::MAX;
        for day in state.get_schedule_matrix() {
            let x = day.iter().map(|t| t.count_total()).sum::<usize>();
            max_sessions = max_sessions.max(x);
            min_sessions = min_sessions.min(x);
        }
        (max_sessions - min_sessions) as f32 * self.weight
    }
}

pub struct SessionLengthLimits {
    weight: f32,
    min_len: usize,
    max_len: usize,
}

impl SessionLengthLimits {
    pub fn new(weight: f32, min_len: usize, max_len: usize) -> Self {
        Self {
            weight,
            min_len,
            max_len,
        }
    }
}

impl Evaluator for SessionLengthLimits {
    fn evaluate(&self, state: &CalendarState) -> f32 {
        let mut count = 0;
        for (_class_id, class_schedule) in state.get_class_schedules() {
            for day in class_schedule {
                for (_key, group) in &Itertools::group_by(day.iter(), |x| **x > 0) {
                    let group_len = group.count();
                    if group_len > self.max_len || group_len < self.min_len {
                        count += 1;
                    }
                }
            }
        }
        count as f32 * self.weight
    }
}
