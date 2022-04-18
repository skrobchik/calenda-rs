use crate::calendars::CalendarState;
use itertools::Itertools;

pub enum ParameterValue<'a> {
    F32(&'a mut f32),
    Usize(&'a mut usize)
}

pub struct EvaluatorParameter<'a> {
    pub name: &'a str,
    pub value: ParameterValue<'a>,
}

impl<'a> EvaluatorParameter<'a> {
    fn from_f32(value: &'a mut f32, name: &'a str) -> Self {
        Self {
            name,
            value: ParameterValue::F32(value),
        }
    }
    fn from_usize(value: &'a mut usize, name: &'a str) -> Self {
        Self {
            name,
            value: ParameterValue::Usize(value),
        }
    }
}

pub struct GapCount {
    weight: f32,
}

impl GapCount {
    pub fn new(weight: f32) -> Self {
        GapCount { weight }
    }
}

pub trait Evaluator {
    fn get_name(&self) -> &str;
    fn get_parameters_mut(&mut self) -> Vec<EvaluatorParameter>;
    fn evaluate(&self, state: &CalendarState) -> f32;
}

impl Evaluator for GapCount {
    fn get_name(&self) -> &str {
        "Gap Count"
    }
    fn get_parameters_mut(&mut self) -> Vec<EvaluatorParameter> {
        vec![
            EvaluatorParameter::from_f32(&mut self.weight, "Weight"),
        ]
    }
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
    fn get_name(&self) -> &str {
        "Daylight"
    }
    fn get_parameters_mut(&mut self) -> Vec<EvaluatorParameter> {
        vec![
            EvaluatorParameter::from_f32(&mut self.weight, "Weight"),
            EvaluatorParameter::from_usize(&mut self.wake_up_time, "Wake up time"),
            EvaluatorParameter::from_usize(&mut self.sleep_time, "Sleep time")
        ]
    }
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
    fn get_name(&self) -> &str {
        "Colliding"
    }
    fn get_parameters_mut(&mut self) -> Vec<EvaluatorParameter> {
        vec![
            EvaluatorParameter::from_f32(&mut self.weight, "Weight"),
        ]
    }
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
    fn get_name(&self) -> &str {
        "Daily Work Difference"
    }
    fn get_parameters_mut(&mut self) -> Vec<EvaluatorParameter> {
        vec![
            EvaluatorParameter::from_f32(&mut self.weight, "Weight"),
        ]
    }
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
    fn get_name(&self) -> &str {
        "Session Length"
    }
    fn get_parameters_mut(&mut self) -> Vec<EvaluatorParameter> {
        vec![
            EvaluatorParameter::from_f32(&mut self.weight, "Weight"),
            EvaluatorParameter::from_usize(&mut self.min_len, "Minimum Length"),
            EvaluatorParameter::from_usize(&mut self.max_len, "Maximum Length")
        ]
    }
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
