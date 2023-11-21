use std::{
  collections::BTreeMap,
  sync::{Arc, Mutex},
  thread::{self, JoinHandle},
};

use indicatif::{ProgressIterator, ProgressStyle};
use num::Integer;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
  heuristics,
  school_schedule::{
    class_calendar::{ClassCalendar, ClassEntryDelta},
    SimulationConstraints,
  },
  timeslot,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum TemperatureFunction {
  T001,
  T002,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ScheduleGenerationOptions {
  pub(crate) simulation_constraints: SimulationConstraints,
  pub(crate) steps: usize,
  pub(crate) parallel_count: usize,
  pub(crate) initial_state: Option<ClassCalendar>,
  #[serde(skip)]
  pub(crate) multi_progress: Option<Arc<Mutex<indicatif::MultiProgress>>>,
  pub(crate) temperature_function: TemperatureFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ScheduleGenerationOutput {
  pub(crate) best_schedule: ClassCalendar,
  pub(crate) best_schedule_cost: f64,
  pub(crate) best_schedule_run_report: SimulationRunReport,
}

pub(crate) fn generate_schedule(
  options: ScheduleGenerationOptions,
) -> JoinHandle<ScheduleGenerationOutput> {
  if options.initial_state.is_some() {
    unimplemented!()
  }
  thread::spawn(move || {
    // let n = std::thread::available_parallelism()
    //   .map_or(1, |x| x.into())
    //   .div_ceil(&2_usize);
    let constraints = Arc::new(options.simulation_constraints);
    let handles: Vec<JoinHandle<(ClassCalendar, f64, SimulationRunReport)>> = (0..options
      .parallel_count)
      .map(|_i: usize| {
        let local_constraints = constraints.clone();
        let local_multi_progress = options.multi_progress.clone();
        let local_temperature_function = options.temperature_function.clone();
        thread::spawn(move || {
          simulated_annealing(&local_constraints, options.steps, local_multi_progress, &local_temperature_function)
        })
      })
      .collect();
    let results: Vec<(ClassCalendar, f64, SimulationRunReport)> =
      handles.into_iter().map(|h| h.join().unwrap()).collect();
    let best_result = results
      .into_iter()
      .min_by(|x, y| x.1.partial_cmp(&y.1).unwrap())
      .unwrap();
    ScheduleGenerationOutput {
      best_schedule: best_result.0,
      best_schedule_cost: best_result.1,
      best_schedule_run_report: best_result.2,
    }
  })
}

#[derive(thiserror::Error, Debug)]
pub enum StatsTrackerError {
  #[error("You are logging the same stat twice per step `{0}`")]
  MultiStatLogging(String),
  #[error("You missed logging this stat in a step `{0}`")]
  MissedStatLogging(String),
}

#[derive(Serialize, Deserialize)]
struct StatsTracker {
  step_index: usize,
  stats_index: usize,
  sampling_rate: usize,
  stats: BTreeMap<String, Vec<serde_json::Value>>,
  is_logging_step: bool,
}

impl StatsTracker {
  fn new(sampling_rate: usize) -> Self {
    StatsTracker {
      step_index: 0,
      stats_index: 0,
      sampling_rate,
      stats: Default::default(),
      is_logging_step: true,
    }
  }

  fn into_stats(self) -> BTreeMap<String, Vec<serde_json::Value>> {
    self.stats
  }

  fn inc_step(&mut self) {
    self.step_index += 1;
    if self.step_index % self.sampling_rate == 0 {
      self.is_logging_step = true;
      self.stats_index += 1;
    } else {
      self.is_logging_step = false;
    }
  }

  fn log_stat<T: Into<serde_json::Value>>(
    &mut self,
    stat_label: &str,
    stat_value: T,
  ) -> Result<(), StatsTrackerError> {
    if !self.is_logging_step {
      return Ok(());
    }
    let stat_value: serde_json::Value = stat_value.into();
    let stat_vector = self.stats.entry(stat_label.into()).or_default();
    match stat_vector.len().cmp(&self.stats_index) {
      std::cmp::Ordering::Less => Err(StatsTrackerError::MissedStatLogging(stat_label.into())),
      std::cmp::Ordering::Equal => {
        stat_vector.push(stat_value);
        Ok(())
      }
      std::cmp::Ordering::Greater => Err(StatsTrackerError::MultiStatLogging(stat_label.into())),
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SimulationRunReport {
  pub(crate) num_steps: u64,
  pub(crate) stats: BTreeMap<String, Vec<serde_json::Value>>,
  pub(crate) start_time: std::time::SystemTime,
}

fn simulated_annealing(
  constraints: &SimulationConstraints,
  steps: usize,
  multi_progress: Option<Arc<Mutex<indicatif::MultiProgress>>>,
  temperature_function: &TemperatureFunction,
) -> (ClassCalendar, f64, SimulationRunReport) {
  // let seed: [u8; 32] = "Aritz123Aritz123Aritz123Aritz123"
  //   .as_bytes()
  //   .try_into()
  //   .unwrap();
  // let mut rng = rand::rngs::StdRng::from_seed(seed);
  let mut rng = rand::rngs::ThreadRng::default();

  let mut stats_tracker = StatsTracker::new(steps.div_ceil(&5_000) as usize);

  let mut state = random_init(constraints, &mut rng);
  let mut state_cost = cost(&state, constraints);

  let progress_bar_style = ProgressStyle::with_template(
    "{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {human_pos}/{human_len} ({percent} %) ({eta}) ({per_sec})",
  )
  .unwrap()
  .progress_chars("#>-");

  let progress_bar = {
    let pb = indicatif::ProgressBar::new(steps as u64).with_style(progress_bar_style);
    if let Some(multi_progress) = multi_progress {
      let multi_progress = multi_progress.lock().unwrap();
      multi_progress.add(pb)
    } else {
      pb
    }
  };

  for step in 0..steps {
    stats_tracker.log_stat("curr_cost", state_cost).unwrap();
    let t = {
      let x = ((step + 1) as f64) / (steps as f64);
      stats_tracker.log_stat("x", x).unwrap();
      let t = temperature(x, temperature_function);
      stats_tracker.log_stat("temperature", t).unwrap();
      t
    };
    let old_cost = state_cost;
    let delta = state.move_one_class_random(&mut rng);

    let new_cost = cost(&state, constraints);

    stats_tracker.log_stat("new_cost", new_cost).unwrap();

    let ap = acceptance_probability(old_cost, new_cost, t);
    stats_tracker
      .log_stat("acceptance_probability", ap)
      .unwrap();

    if ap >= rng.gen_range(0.0..=1.0) {
      stats_tracker.log_stat("accepted", true).unwrap();
      // keep change
      state_cost = new_cost;
    } else {
      stats_tracker.log_stat("accepted", false).unwrap();
      revert_change(&mut state, &delta);
      state_cost = old_cost;
    }

    stats_tracker.inc_step();
    progress_bar.inc(1);
  }

  let run_report = SimulationRunReport {
    num_steps: steps as u64,
    stats: stats_tracker.into_stats(),
    start_time: std::time::SystemTime::now(),
  };

  (state, state_cost, run_report)
}

fn acceptance_probability(old_cost: f64, new_cost: f64, temperature: f64) -> f64 {
  if new_cost < old_cost {
    1.0
  } else {
    (-(new_cost - old_cost) / temperature.max(f64::EPSILON)).exp()
  }
}

fn temperature(x: f64, temperature_function_variant: &TemperatureFunction) -> f64 {
  // 10.0 - 10.0 * x

  // if x <= 0.9 {
  //   9.0 - 10.0 * x
  // } else {
  //   0.0
  // }
  match temperature_function_variant {
    TemperatureFunction::T001 => {
      if x <= 0.8 {
        4.0 - 5.0 * x
      } else {
        0.0
      }
    },
    TemperatureFunction::T002 => {
      if x <= 0.9 { 7.5*(0.5*(1.1*7.0*std::f64::consts::PI*x+std::f64::consts::FRAC_2_PI).sin()+0.5) } else { 0.0 }
    },
}
  

  // 0.0
  // 7.5*(0.5*(5.0*std::f64::consts::PI*x+std::f64::consts::FRAC_2_PI).sin()+0.5)
  // if x <= 0.9 { 7.5*(0.5*(1.1*7.0*std::f64::consts::PI*x+std::f64::consts::FRAC_2_PI).sin()+0.5) } else { 0.0 }
}

fn random_init<R: Rng>(constraints: &SimulationConstraints, rng: &mut R) -> ClassCalendar {
  let mut state: ClassCalendar = Default::default();

  for (class_id, class) in constraints.get_classes().iter().enumerate() {
    for _ in 0..*class.get_class_hours() {
      let timeslot_idx = rng.gen_range(timeslot::TIMESLOT_RANGE);
      let day_idx = rng.gen_range(timeslot::DAY_RANGE);
      state.add_one_class(day_idx, timeslot_idx, class_id)
    }
  }

  state
}

fn revert_change(state: &mut ClassCalendar, delta: &ClassEntryDelta) {
  state.move_one_class(
    delta.dst_day_idx,
    delta.dst_timeslot_idx,
    delta.src_day_idx,
    delta.src_timeslot_idx,
    delta.class_id,
  );
}

fn cost(state: &ClassCalendar, constraints: &SimulationConstraints) -> f64 {
  0.0
    + 5.0 * heuristics::same_timeslot_classes_count_per_semester(state, constraints)
    + 10.0 * heuristics::same_timeslot_classes_count_per_professor(state, constraints)
    + 3.0 * heuristics::count_not_available(state, constraints)
    + 1.0 * heuristics::count_available_if_needed(state, constraints)
    + 1.0 * heuristics::count_outside_session_length(state, 2, 4)
    + 1.0 * heuristics::count_inconsistent_class_timeslots(state)
}
