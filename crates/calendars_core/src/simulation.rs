use std::{
  collections::BTreeMap,
  thread::{self, JoinHandle},
};

use crate::{
  school_schedule::{Classroom, ClassroomAssignmentKey, ClassroomType},
  simulation_options::{ProgressOption, SimulationOptions, StopCondition, TemperatureFunction},
  stats_tracker::StatsTracker,
};
use indicatif::{HumanCount, HumanDuration, ProgressStyle};
use itertools::Itertools;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use strum::{IntoEnumIterator, VariantArray};

use crate::{
  heuristics,
  school_schedule::{
    class_calendar::{ClassCalendar, ClassEntryDelta},
    SimulationConstraints,
  },
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimulationOutput {
  pub simulation_options: SimulationOptions,
  pub final_calendar: ClassCalendar,
  pub final_cost: f64,
  pub start_time: std::time::SystemTime,
  pub end_time: std::time::SystemTime,

  /// Not necesarilly equal to `end_time - start_time` (e.g., the system time was changed during the simulation run).
  pub duration: std::time::Duration,

  pub stats: BTreeMap<String, Vec<serde_json::Value>>,

  /// Needed if stop condition is not number of steps
  pub total_steps: usize,

  #[serde(skip)]
  pub classroom_assignments: BTreeMap<ClassroomAssignmentKey, Classroom>,
}

pub fn generate_schedule(
  options_list: Vec<SimulationOptions>,
  seed: Option<u64>,
) -> JoinHandle<Vec<SimulationOutput>> {
  thread::spawn(move || {
    let rng = match seed {
      Some(seed) => ChaCha8Rng::seed_from_u64(seed),
      None => ChaCha8Rng::from_entropy(),
    };

    let handles: Vec<JoinHandle<SimulationOutput>> = options_list
      .into_iter()
      .enumerate()
      .map(|(simulation_idx, options)| {
        let mut rng = rng.clone();
        rng.set_stream(simulation_idx as u64);
        thread::spawn(move || simulated_annealing(options, rng))
      })
      .collect();

    let results: Vec<SimulationOutput> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    results
  })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimulationRunReport {
  pub num_steps: u64,
  pub stats: BTreeMap<String, Vec<serde_json::Value>>,
  pub start_time: std::time::SystemTime,
}

fn simulated_annealing<R: Rng>(options: SimulationOptions, mut rng: R) -> SimulationOutput {
  let start_time = std::time::SystemTime::now();
  let start_instant = std::time::Instant::now();

  let constraints: &SimulationConstraints = &options.simulation_constraints;
  let stop_condition = &options.stop_condition;
  let temperature_function = &options.temperature_function;

  let mut stats = StatsTracker::with_estimated_size(stop_condition, 5_000);

  // let mut state = random_init(constraints, &mut rng);
  let mut state = options.initial_state.clone();
  let mut state_cost = cost(&state, constraints);

  let mut progress_bar: Option<indicatif::ProgressBar> = {
    match (options.progress, &options.stop_condition) {
      (ProgressOption::ProgressBar(pb), StopCondition::Steps(total_steps)) => {
        pb.set_length(*total_steps as u64);
        pb.set_position(0);
        Some(pb)
      }
      (ProgressOption::MultiProgress(mp), StopCondition::Steps(total_steps)) => {
        let progress_bar_style = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {human_pos}/{human_len} ({percent} %) ({eta}) ({per_sec})",
          )
          .unwrap()
          .progress_chars("#>-");
        let pb = indicatif::ProgressBar::new(*total_steps as u64).with_style(progress_bar_style);
        let pb = mp.add(pb);
        Some(pb)
      }
      (ProgressOption::MultiProgress(mp), StopCondition::Time(total_duration)) => {
        let human_total_time = HumanDuration(*total_duration);
        let pb_style = ProgressStyle::with_template(
          format!(
            "{{spinner:.green}} [{{bar:.cyan/blue}}] ({{percent}} %) [{{elapsed}}/{}] ({{msg}} steps)",
            human_total_time
          )
          .as_str(),
        )
        .unwrap()
        .progress_chars("#>-");
        let pb = indicatif::ProgressBar::new(total_duration.as_secs())
          .with_style(pb_style)
          .with_message("0");
        let pb = mp.add(pb);
        Some(pb)
      }
      (ProgressOption::ProgressBar(pb), StopCondition::Time(total_duration)) => {
        pb.set_length(total_duration.as_secs());
        pb.set_position(0);
        Some(pb)
      }
      (ProgressOption::None, _) => None,
    }
  };

  let mut step_idx = 0;
  while match stop_condition {
    StopCondition::Steps(total_steps) => step_idx < *total_steps,
    StopCondition::Time(total_time) => start_instant.elapsed().lt(total_time),
  } {
    stats.log_stat("curr_cost", state_cost).unwrap();

    let x = match stop_condition {
      StopCondition::Steps(total_steps) => ((step_idx + 1) as f64) / (*total_steps as f64),
      StopCondition::Time(total_time) => {
        (start_instant.elapsed().as_nanos() / total_time.as_nanos()).max(1) as f64
      }
    };
    stats.log_stat("x", x).unwrap();

    let t_amplitude = 1.0;
    let t = temperature(x, temperature_function, t_amplitude);
    stats.log_stat("temperature", t).unwrap();

    let old_cost = state_cost;
    let delta = state.move_one_class_random(&mut rng).unwrap();

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
    if step_idx % options.advanced_options.progress_bar_update_interval == 0 {
      match (progress_bar.as_mut(), stop_condition) {
        (Some(pb), StopCondition::Steps(_)) => {
          pb.set_position(step_idx as u64);
        }
        (Some(pb), StopCondition::Time(_)) => {
          let human_steps = HumanCount(step_idx as u64);
          pb.set_position(start_instant.elapsed().as_secs());
          pb.set_message(human_steps.to_string());
        }
        _ => (),
      };
    }
    if let Some(live_update) = options.advanced_options.live_update.as_ref() {
      if step_idx % live_update.live_update_interval == 0 {
        if let Ok(()) = live_update.channel.send(state.clone()) {}
      }
    }
    step_idx += 1;
  }

  let end_time = std::time::SystemTime::now();
  let duration = start_instant.elapsed();

  let classroom_assignments = assign_classrooms(&state, constraints);
  SimulationOutput {
    simulation_options: SimulationOptions {
      simulation_constraints: options.simulation_constraints,
      initial_state: options.initial_state,
      progress: ProgressOption::None,
      temperature_function: options.temperature_function,
      advanced_options: Default::default(),
      stop_condition: options.stop_condition,
    },
    total_steps: step_idx,
    final_calendar: state,
    final_cost: state_cost,
    start_time,
    end_time,
    duration,
    stats: stats.into_stats(),
    classroom_assignments,
  }
}

fn acceptance_probability(old_cost: f64, new_cost: f64, temperature: f64) -> f64 {
  if new_cost < old_cost {
    1.0
  } else {
    (-(new_cost - old_cost) / temperature.max(f64::EPSILON)).exp()
  }
}

fn temperature(x: f64, temperature_function_variant: &TemperatureFunction, amplitude: f64) -> f64 {
  (match temperature_function_variant {
    TemperatureFunction::Linear => 1.0 - x,
  })
  .clamp(0.0, 1.0)
    * amplitude
}

fn revert_change(state: &mut ClassCalendar, delta: &ClassEntryDelta) {
  state.move_one_class(
    delta.dst_day,
    delta.dst_timeslot,
    delta.src_day,
    delta.src_timeslot,
    delta.class_key,
  );
}

fn cost(state: &ClassCalendar, constraints: &SimulationConstraints) -> f64 {
  0.0
    + 10.0 * (count_classroom_assignment_collisions(state, constraints) as f64)
    + 9.0 * heuristics::same_timeslot_classes_count_per_professor(state, constraints)
    + 5.0 * heuristics::same_timeslot_classes_count_per_semester(state, constraints)
    + 4.5 * heuristics::count_labs_on_different_days(state, constraints)
    + 3.0 * heuristics::count_not_available(state, constraints)
    + 2.5 * heuristics::count_incontinuous_classes(state)
    + 1.5 * heuristics::count_outside_session_length(state, 2, 4)
    + 1.25 * heuristics::count_available_if_needed(state, constraints)
    + 1.0 * heuristics::count_inconsistent_class_timeslots(state)
    + 0.1 * heuristics::same_timeslot_classes_count(state)
}

pub(crate) fn assign_classrooms(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> BTreeMap<ClassroomAssignmentKey, Classroom> {
  let matching = assign_classrooms_matching(state, constraints);
  matching
    .into_iter()
    .map(|(a, b)| match (a, b) {
      (
        ClassroomAssignmentVertex::ClassSession(classroom_assignment_key),
        ClassroomAssignmentVertex::Classroom(classroom),
      ) => (classroom_assignment_key, classroom),
      _ => unreachable!(),
    })
    .collect()
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
enum ClassroomAssignmentVertex {
  ClassSession(ClassroomAssignmentKey),
  Classroom(Classroom),
}

fn assign_classrooms_matching(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> Vec<(ClassroomAssignmentVertex, ClassroomAssignmentVertex)> {
  let available_classrooms: [Vec<Classroom>; ClassroomType::VARIANTS.len()] =
    ClassroomType::VARIANTS
      .iter()
      .map(|v| {
        Classroom::iter()
          .filter(|c| c.get_type() == *v)
          .collect_vec()
      })
      .collect_vec()
      .try_into()
      .unwrap();

  let mut edges: Vec<(ClassroomAssignmentVertex, ClassroomAssignmentVertex)> = Vec::new();
  for entry in state.get_entries() {
    let class = constraints.get_class(entry.class_key).unwrap();
    let classroom_assignment_key = ClassroomAssignmentKey {
      day: entry.day,
      timeslot: entry.timeslot,
      class_key: entry.class_key,
    };
    let entry_edges = class
      .get_allowed_classroom_types()
      .iter()
      .map(|classroom_type| {
        ClassroomType::VARIANTS
          .iter()
          .position(|v| *v == classroom_type)
          .unwrap()
      })
      .flat_map(|classroom_type_i| &available_classrooms[classroom_type_i])
      .unique()
      .map(|classroom| (classroom_assignment_key, *classroom))
      .map(|(a, b)| {
        (
          ClassroomAssignmentVertex::ClassSession(a),
          ClassroomAssignmentVertex::Classroom(b),
        )
      });
    edges.extend(entry_edges);
  }

  hopcroft_karp::matching(&edges)
}

fn count_classroom_assignment_collisions(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> usize {
  state
    .get_entries()
    .len()
    .checked_sub(assign_classrooms_matching(state, constraints).len())
    .expect("Can't be more matching than class entries")
}
