use std::{
  collections::BTreeMap,
  thread::{self, JoinHandle},
};

use crate::{
  class_filter::ClassFilter,
  school_schedule::{Classroom, ClassroomAssignmentKey, ClassroomType},
  simulation_options::{ProgressOption, SimulationOptions, StopCondition, TemperatureFunction},
  stats_tracker::StatsTracker,
  timeslot::{DAY_RANGE, TIMESLOT_RANGE},
};
use indicatif::{HumanCount, HumanDuration, ProgressStyle};
use itertools::Itertools;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::{
  heuristics,
  school_schedule::{
    class_calendar::{ClassCalendar, ClassEntryDelta},
    SimulationConstraints,
  },
  timeslot,
};

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

  /// Needed if stop condition is not number of steps
  pub(crate) total_steps: usize,

  pub(crate) classroom_assignments: BTreeMap<ClassroomAssignmentKey, Classroom>,
}

pub(crate) fn generate_schedule(
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
pub(crate) struct SimulationRunReport {
  pub(crate) num_steps: u64,
  pub(crate) stats: BTreeMap<String, Vec<serde_json::Value>>,
  pub(crate) start_time: std::time::SystemTime,
}

fn simulated_annealing<R: Rng>(options: SimulationOptions, mut rng: R) -> SimulationOutput {
  let start_time = std::time::SystemTime::now();
  let start_instant = std::time::Instant::now();

  let constraints: &SimulationConstraints = &options.simulation_constraints;
  let stop_condition = &options.stop_condition;
  let temperature_function = &options.temperature_function;

  let mut stats = StatsTracker::with_estimated_size(stop_condition, 5_000);

  let mut state = random_init(constraints, &mut rng);
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
    + 9.0 * heuristics::same_timeslot_classes_count_per_professor(state, constraints)
    + 10.0 * (count_classroom_assignment_collisions(state, constraints) as f64)
    + 3.0 * heuristics::count_not_available(state, constraints)
    + 1.1 * heuristics::count_available_if_needed(state, constraints)
    + 2.0 * heuristics::count_outside_session_length(state, 1, 3)
    + 1.5 * heuristics::count_outside_session_length(state, 2, 2)
    + 1.0 * heuristics::count_inconsistent_class_timeslots(state)
    + 0.1 * heuristics::same_timeslot_classes_count(state, constraints, &ClassFilter::None)
}

fn assign_classrooms(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> BTreeMap<ClassroomAssignmentKey, Classroom> {
  let available_classrooms: [Vec<Classroom>; enum_iterator::cardinality::<ClassroomType>()] = {
    const EMPTY_VEC: Vec<Classroom> = Vec::new();
    let mut available_classrooms = [EMPTY_VEC; enum_iterator::cardinality::<ClassroomType>()];
    for classroom in enum_iterator::all::<Classroom>() {
      available_classrooms[classroom.get_type() as usize].push(classroom);
    }
    available_classrooms
  };
  let default_classroom: [Classroom; enum_iterator::cardinality::<ClassroomType>()] =
    available_classrooms
      .iter()
      .map(|v| {
        v.get(0)
          .expect("A classroom type doesn't have a classroom")
          .clone()
      })
      .collect_vec()
      .try_into()
      .unwrap();
  let mut classroom_assignment: BTreeMap<ClassroomAssignmentKey, Classroom> = BTreeMap::new();
  for day_idx in DAY_RANGE {
    for timeslot_idx in TIMESLOT_RANGE {
      let mut timeslot_available_classrooms = available_classrooms.clone();
      for (class_id, count) in state.get_timeslot(day_idx, timeslot_idx).iter().enumerate() {
        if *count == 0 {
          continue;
        }
        // if class is repeating (`count` >= 2) it will be assigned the same classroom, but at this point
        // the schedule is really not very good so it doesn't matter
        let required_classroom_type = *constraints.get_classes()[class_id].get_classroom_type();
        if timeslot_available_classrooms[required_classroom_type as usize]
          .last()
          .is_some()
        {
          classroom_assignment.insert(
            ClassroomAssignmentKey {
              day_idx,
              timeslot_idx,
              class_id,
            },
            timeslot_available_classrooms[required_classroom_type as usize]
              .pop()
              .unwrap(),
          );
        } else {
          classroom_assignment.insert(
            ClassroomAssignmentKey {
              day_idx,
              timeslot_idx,
              class_id,
            },
            default_classroom[required_classroom_type as usize].clone(),
          );
        }
      }
    }
  }
  classroom_assignment
}

fn count_classroom_assignment_collisions(
  state: &ClassCalendar,
  constraints: &SimulationConstraints,
) -> u32 {
  let available_classrooms: [Vec<Classroom>; enum_iterator::cardinality::<ClassroomType>()] = {
    const EMPTY_VEC: Vec<Classroom> = Vec::new();
    let mut available_classrooms = [EMPTY_VEC; enum_iterator::cardinality::<ClassroomType>()];
    for classroom in enum_iterator::all::<Classroom>() {
      available_classrooms[classroom.get_type() as usize].push(classroom);
    }
    available_classrooms
  };
  let mut num_classroom_assignment_collisions = 0;
  for day_idx in DAY_RANGE {
    for timeslot_idx in TIMESLOT_RANGE {
      let mut timeslot_available_classrooms = available_classrooms.clone();
      for (class_id, count) in state.get_timeslot(day_idx, timeslot_idx).iter().enumerate() {
        for _ in 0..*count {
          let required_classroom_type = *constraints.get_classes()[class_id].get_classroom_type();
          if timeslot_available_classrooms[required_classroom_type as usize]
            .last()
            .is_some()
          {
            timeslot_available_classrooms[required_classroom_type as usize]
              .pop()
              .unwrap();
          } else {
            num_classroom_assignment_collisions += 1;
          }
        }
      }
    }
  }
  num_classroom_assignment_collisions
}
