use std::{
  collections::BTreeMap,
  thread::{self, JoinHandle},
};

use crate::stats_tracker::StatsTracker;
use indicatif::ProgressStyle;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
  heuristics,
  school_schedule::{
    class_calendar::{ClassCalendar, ClassEntryDelta},
    SimulationConstraints,
  },
  timeslot,
};

#[derive(Debug, Clone, Default)]
pub(crate) enum ProgressOption {
  ProgressBar(indicatif::ProgressBar),
  MultiProgress(indicatif::MultiProgress),
  #[default]
  None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum TemperatureFunction {
  T001,
  T002,
  T003,
  T004,
}

const _: () = {
  fn assert_send<T: Send>() {}
  fn assert_sync<T: Sync>() {}
  fn assert_all() {
    assert_send::<SimulationOptions>();
    assert_sync::<SimulationOptions>();
  }
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SimulationOptions {
  pub(crate) simulation_constraints: SimulationConstraints,
  pub(crate) total_steps: usize,
  pub(crate) initial_state: Option<ClassCalendar>,
  #[serde(skip)]
  pub(crate) progress: ProgressOption,
  pub(crate) temperature_function: TemperatureFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SimulationOutput {
  pub(crate) simulation_options: SimulationOptions,
  pub(crate) final_calendar: ClassCalendar,
  pub(crate) final_cost: f64,
  pub(crate) start_time: std::time::SystemTime,
  pub(crate) end_time: std::time::SystemTime,

  /// Not necesarilly equal to `end_time - start_time` (e.g., the system time was changed during the simulation run).
  pub(crate) duration: std::time::Duration,

  pub(crate) stats: BTreeMap<String, Vec<serde_json::Value>>,
}

pub(crate) fn generate_schedule(
  options_list: Vec<SimulationOptions>,
  _seed: Option<u64>,
) -> JoinHandle<Vec<SimulationOutput>> {
  thread::spawn(move || {
    let handles: Vec<JoinHandle<SimulationOutput>> = options_list
      .into_iter()
      .map(|options| thread::spawn(move || simulated_annealing(options, thread_rng())))
      .collect();

    let results: Vec<SimulationOutput> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    results
  })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SimulationRunReport {
  pub(crate) num_steps: u64,
  pub(crate) stats: BTreeMap<String, Vec<serde_json::Value>>,
  pub(crate) start_time: std::time::SystemTime,
}

fn simulated_annealing<R: Rng>(options: SimulationOptions, mut rng: R) -> SimulationOutput {
  let start_time = std::time::SystemTime::now();
  let start_instant = std::time::Instant::now();

  let constraints = &options.simulation_constraints;
  let total_steps = &options.total_steps;
  let temperature_function = &options.temperature_function;

  let mut stats = StatsTracker::new(total_steps.div_ceil(5_000_usize));

  let mut state = random_init(constraints, &mut rng);
  let mut state_cost = cost(&state, constraints);

  let mut progress_bar: Option<indicatif::ProgressBar> = {
    match options.progress {
      ProgressOption::ProgressBar(pb) => Some(pb),
      ProgressOption::MultiProgress(mp) => {
        let progress_bar_style = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {human_pos}/{human_len} ({percent} %) ({eta}) ({per_sec})",
          )
          .unwrap()
          .progress_chars("#>-");
        let pb = indicatif::ProgressBar::new(*total_steps as u64).with_style(progress_bar_style);
        let pb = mp.add(pb);
        Some(pb)
      }
      ProgressOption::None => None,
    }
  };

  for step in 0..*total_steps {
    stats.log_stat("curr_cost", state_cost).unwrap();
    let t = {
      let x = ((step + 1) as f64) / (*total_steps as f64);
      stats.log_stat("x", x).unwrap();
      let t = temperature(x, temperature_function);
      stats.log_stat("temperature", t).unwrap();
      t
    };
    let old_cost = state_cost;
    let delta = state.move_one_class_random(&mut rng);

    let new_cost = cost(&state, constraints);

    stats.log_stat("new_cost", new_cost).unwrap();

    let ap = acceptance_probability(old_cost, new_cost, t);
    stats.log_stat("acceptance_probability", ap).unwrap();

    if ap >= rng.gen_range(0.0..=1.0) {
      stats.log_stat("accepted", true).unwrap();
      // keep change
      state_cost = new_cost;
    } else {
      stats.log_stat("accepted", false).unwrap();
      revert_change(&mut state, &delta);
      state_cost = old_cost;
    }

    stats.inc_step();
    if let Some(pb) = progress_bar.as_mut() {
      pb.inc(1)
    }
  }

  let end_time = std::time::SystemTime::now();
  let duration = start_instant.elapsed();

  SimulationOutput {
    simulation_options: SimulationOptions {
      simulation_constraints: options.simulation_constraints,
      total_steps: options.total_steps,
      initial_state: options.initial_state,
      progress: ProgressOption::None,
      temperature_function: options.temperature_function,
    },
    final_calendar: state,
    final_cost: state_cost,
    start_time,
    end_time,
    duration,
    stats: stats.into_stats(),
  }
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
    }
    TemperatureFunction::T002 => {
      if x <= 0.9 {
        7.5
          * (0.5 * (1.1 * 7.0 * std::f64::consts::PI * x + std::f64::consts::FRAC_2_PI).sin() + 0.5)
      } else {
        0.0
      }
    }
    TemperatureFunction::T003 => {
      if x <= 0.9 {
        (1.0 - x)
          * 10.0
          * (0.5 * (1.1 * 7.0 * std::f64::consts::PI * x + std::f64::consts::FRAC_2_PI).sin() + 0.5)
      } else {
        0.0
      }
    }
    TemperatureFunction::T004 => {
      if x <= 0.9 {
        (1.0 - x)
          * 5.0
          * (0.5 * (1.1 * 7.0 * std::f64::consts::PI * x + std::f64::consts::FRAC_2_PI).sin() + 0.5)
      } else {
        0.0
      }
    }
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
