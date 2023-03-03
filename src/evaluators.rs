use std::collections::{BTreeMap, BTreeSet};

use crate::metadata_register::MetadataRegister;
use crate::{calendars::CalendarState, metadata_register::SemesterNumber, timeslot::DAY_RANGE};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub enum Evaluator {
  GapCount {
    weight: f32,
  },
  Daylight {
    weight: f32,
    wake_up_time: usize,
    sleep_time: usize,
  },
  Colliding {
    weight: f32,
  },
  DailyWorkDifference {
    weight: f32,
  },
  SessionLengthLimits {
    weight: f32,
    min_len: usize,
    max_len: usize,
  },
  ClassSeparation {
    weight: f32,
  },
}

fn eval_class_separation(
  weight: f32,
  state: &CalendarState,
  metadata_register: &MetadataRegister,
) -> f32 {
  let mut separated_groups = 0;
  for (_class_id, class_schedule) in state.get_class_schedules() {
    for day in DAY_RANGE {
      let day_array = class_schedule[day];
      let mut groups = 0;
      for (key, _group) in &day_array.iter().group_by(|x| **x > 0) {
        if key {
          groups += 1;
        }
      }
      if groups > 0 {
        groups -= 1;
      }
      separated_groups += groups;
    }
  }
  weight * separated_groups as f32
}

fn eval_gap_count(weight: f32, state: &CalendarState, metadata_register: &MetadataRegister) -> f32 {
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
  count as f32 * weight
}

fn eval_daylight(
  weight: f32,
  wake_up_time: usize,
  sleep_time: usize,
  state: &CalendarState,
  metadata_register: &MetadataRegister,
) -> f32 {
  state
    .get_session_set()
    .iter()
    .filter(|(session, _count)| {
      session.t.timeslot < wake_up_time || session.t.timeslot >= sleep_time
    })
    .map(|(_session, count)| count)
    .map(|x| x)
    .sum::<usize>() as f32
    * weight
}

fn eval_colliding(weight: f32, state: &CalendarState, metadata_register: &MetadataRegister) -> f32 {
  let e = state
    .get_schedule_matrix()
    .iter()
    .flatten()
    .map(|x| {
      let mut collisions = 0;
      let mut seen: BTreeSet<SemesterNumber> = BTreeSet::new();
      for (class_id, count) in x.iter() {
        let semester = metadata_register
          .get_class_metadata(*class_id)
          .unwrap()
          .semester_number;
        let key = (semester, *class_id);
        for _ in 0..*count {
          if seen.contains(&semester) {
            collisions += 1;
          } else {
            seen.insert(semester);
          }
        }
      }
      collisions as f32
    })
    .sum::<f32>()
    * weight;
  println!("{}", e);
  e
}

fn eval_daily_work_difference(
  weight: f32,
  state: &CalendarState,
  metadata_register: &MetadataRegister,
) -> f32 {
  let mut max_sessions = 0;
  let mut min_sessions = usize::MAX;
  for day in state.get_schedule_matrix() {
    let x = day.iter().map(|t| t.count_total()).sum::<usize>();
    max_sessions = max_sessions.max(x);
    min_sessions = min_sessions.min(x);
  }
  (max_sessions - min_sessions) as f32 * weight
}

fn eval_session_length_limits(
  weight: f32,
  min_len: usize,
  max_len: usize,
  state: &CalendarState,
  metadata_register: &MetadataRegister,
) -> f32 {
  let mut count = 0;
  for (_class_id, class_schedule) in state.get_class_schedules() {
    for day in class_schedule {
      for (_key, group) in &Itertools::group_by(day.iter(), |x| **x > 0) {
        let group_len = group.count();
        if group_len > max_len || group_len < min_len {
          count += 1;
        }
      }
    }
  }
  count as f32 * weight
}

impl Evaluator {
  pub fn evaluate(&self, state: &CalendarState, metadata_register: &MetadataRegister) -> f32 {
    match self {
      Evaluator::GapCount { weight } => eval_gap_count(*weight, state, metadata_register),
      Evaluator::Daylight {
        weight,
        wake_up_time,
        sleep_time,
      } => eval_daylight(
        *weight,
        *wake_up_time,
        *sleep_time,
        state,
        metadata_register,
      ),
      Evaluator::Colliding { weight } => eval_colliding(*weight, state, metadata_register),
      Evaluator::DailyWorkDifference { weight } => {
        eval_daily_work_difference(*weight, state, metadata_register)
      }
      Evaluator::SessionLengthLimits {
        weight,
        min_len,
        max_len,
      } => eval_session_length_limits(*weight, *min_len, *max_len, state, metadata_register),
      Evaluator::ClassSeparation { weight } => {
        eval_class_separation(*weight, state, metadata_register)
      }
    }
  }
  pub fn get_name(&self) -> &str {
    match self {
      Evaluator::GapCount { weight: _ } => "Gap Count",
      Evaluator::Daylight {
        weight: _,
        wake_up_time: _,
        sleep_time: _,
      } => "Daylight",
      Evaluator::Colliding { weight: _ } => "Colliding",
      Evaluator::DailyWorkDifference { weight: _ } => "Daily Work Difference",
      Evaluator::SessionLengthLimits {
        weight: _,
        min_len: _,
        max_len: _,
      } => "Session Length Limits",
      Evaluator::ClassSeparation { weight: _ } => "Class Separation",
    }
  }
  pub fn get_parameters_mut(&mut self) -> Vec<EvaluatorParameter> {
    match self {
      Evaluator::GapCount { weight } => vec![EvaluatorParameter::from_f32(weight, "Weight")],
      Evaluator::Daylight {
        weight,
        wake_up_time,
        sleep_time,
      } => vec![
        EvaluatorParameter::from_f32(weight, "Weight"),
        EvaluatorParameter::from_usize(wake_up_time, "Wake Up Time"),
        EvaluatorParameter::from_usize(sleep_time, "Sleep Time"),
      ],
      Evaluator::Colliding { weight } => vec![EvaluatorParameter::from_f32(weight, "Weight")],
      Evaluator::DailyWorkDifference { weight } => {
        vec![EvaluatorParameter::from_f32(weight, "Weight")]
      }
      Evaluator::SessionLengthLimits {
        weight,
        min_len,
        max_len,
      } => vec![
        EvaluatorParameter::from_f32(weight, "Weight"),
        EvaluatorParameter::from_usize(min_len, "Min Length"),
        EvaluatorParameter::from_usize(max_len, "Max Length"),
      ],
      Evaluator::ClassSeparation { weight } => vec![EvaluatorParameter::from_f32(weight, "Weight")],
    }
  }
}

pub enum ParameterValue<'a> {
  F32(&'a mut f32),
  Usize(&'a mut usize),
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
