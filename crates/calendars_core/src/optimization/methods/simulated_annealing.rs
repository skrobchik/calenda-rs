use std::{
  collections::BTreeMap,
  rc::Rc,
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Barrier, RwLock,
  },
  thread::{self, JoinHandle},
};

use indicatif::{HumanCount, HumanDuration, ProgressStyle};
use itertools::Itertools;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use strum::{IntoEnumIterator, VariantArray};

use crate::{
  optimization::{class_calendar::ClassEntryDelta, heuristics, stats_tracker::StatsTracker}, school_schedule::ClassroomAssignmentKey, week_calendar, ClassCalendar, ClassCalendarOptimizer, ClassKey, Classroom, ClassroomType, OptimizationConstraints
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimulationRunReport {
  pub num_steps: u64,
  pub stats: BTreeMap<String, Vec<serde_json::Value>>,
  pub start_time: std::time::SystemTime,
}

fn simulated_annealing<R: Rng>(constraints: &OptimizationConstraints, options: SimulationOptions, mut rng: R) -> SimulationOutput {
  let start_time = std::time::SystemTime::now();
  let start_instant = std::time::Instant::now();

  let stop_condition = &options.stop_condition;
  let temperature_function = &options.temperature_function;

  let mut stats = StatsTracker::with_estimated_size(stop_condition, 5_000);

  // let mut state = random_init(constraints, &mut rng);
  let mut state = options.initial_state.clone();
  let mut par_eval = ParEvaluator::new(state.clone(), constraints.clone());
  let mut state_cost = cost(&mut par_eval, &state, constraints);

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

    let t_amplitude = 3.0;
    let t = temperature(x, temperature_function, t_amplitude);
    stats.log_stat("temperature", t).unwrap();

    let old_cost = state_cost;
    let delta = state.move_one_class_random(&mut rng).unwrap();
    par_eval.apply_change(&delta);

    let new_cost = cost(&mut par_eval, &state, constraints);
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
      par_eval.apply_change(&swap_delta(delta));
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

  for (i, evaluator) in EVALUATORS.iter().enumerate() {
    let r = evaluator(&state, constraints);
    let r = r as f64;
    let r = r / (EVALUATORS_FACTOR as f64);
    println!("Evaluator {i}: {r}")
  }
  println!("Total: {}", state_cost);
  let classroom_assignments = assign_classrooms(&state, constraints);
  SimulationOutput {
    simulation_options: SimulationOptions {
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

fn swap_delta(delta: ClassEntryDelta) -> ClassEntryDelta {
  ClassEntryDelta {
    class_key: delta.class_key,
    src_day: delta.dst_day,
    src_timeslot: delta.dst_timeslot,
    dst_day: delta.src_day,
    dst_timeslot: delta.src_timeslot,
  }
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

#[rustfmt::skip]
  const EVALUATORS: [fn(&ClassCalendar, &OptimizationConstraints)->u64; 11] = [
    |state,  constraints| 10000 * (count_classroom_assignment_collisions(state, constraints) as u64),
    |state,  constraints|  9000 * heuristics::same_timeslot_classes_count_per_professor(state, constraints),
    |state,  constraints|  5000 * heuristics::same_timeslot_classes_count_per_semester(state, constraints),
    |state,  constraints|  4500 * heuristics::count_labs_on_different_days(state, constraints),
    |state,  constraints|  3000 * heuristics::count_not_available(state, constraints),
    |state, _constraints|  2500 * heuristics::count_incontinuous_classes(state),
    |state, _constraints|  1500 * heuristics::count_outside_session_length(state, 2, 4),
    |state,  constraints|  1300 * heuristics::count_holes_per_semester(state, constraints),
    |state,  constraints|  1250 * heuristics::count_available_if_needed(state, constraints),
    |state, _constraints|  1000 * heuristics::count_inconsistent_class_timeslots(state),
    |state, _constraints|   100 * heuristics::same_timeslot_classes_count(state),
  ];
const EVALUATORS_FACTOR: u64 = 1000;

fn cost(
  par_eval: &mut ParEvaluator,
  state: &ClassCalendar,
  constraints: &OptimizationConstraints,
) -> f64 {
  let r0 = par_eval.eval_cost();

  #[cfg(debug_assertions)]
  {
    // assert_eq!(state.clone(), par_eval.get_curr_state());

    let r2: u64 = EVALUATORS.iter().map(|f| f(state, constraints)).sum();
    let r2 = r2 as f64;
    let r2 = r2 / (EVALUATORS_FACTOR as f64);

    assert_eq!(r0, r2);
  }

  r0
}

struct ParEvaluator {
  state: Arc<RwLock<ClassCalendar>>,
  start_eval_barrier: Arc<Barrier>,
  finish_eval_barrier: Arc<Barrier>,
  _evaluator_handles: Vec<JoinHandle<()>>,
  cost_counter: Arc<AtomicU64>,
}

impl ParEvaluator {
  fn new(init_state: ClassCalendar, init_constraints: OptimizationConstraints) -> Self {
    let cost_counter = Arc::new(AtomicU64::new(0));
    let state = Arc::new(RwLock::new(init_state));
    let constraints = Arc::new(RwLock::new(init_constraints));
    let start_eval_barrier = Arc::new(Barrier::new(1 + EVALUATORS.len()));
    let finish_eval_barrier = Arc::new(Barrier::new(1 + EVALUATORS.len()));

    let evaluator_handles = (0..EVALUATORS.len())
      .map(|f_i| {
        let local_state = state.clone();
        let local_constraints = constraints.clone();
        let local_start_eval_barrier = start_eval_barrier.clone();
        let local_finish_eval_barrier = finish_eval_barrier.clone();
        let local_cost_counter = cost_counter.clone();
        std::thread::spawn(move || {
          let f = EVALUATORS[f_i];
          loop {
            local_start_eval_barrier.wait();
            let lock_state = local_state.read().unwrap();
            let lock_constraints = local_constraints.read().unwrap();
            local_cost_counter.fetch_add(f(&lock_state, &lock_constraints), Ordering::SeqCst);
            local_finish_eval_barrier.wait();
          }
        })
      })
      .collect_vec();

    Self {
      start_eval_barrier,
      finish_eval_barrier,
      _evaluator_handles: evaluator_handles,
      cost_counter,
      state,
    }
  }

  fn apply_change(&mut self, delta: &ClassEntryDelta) {
    let mut write_lock_state = self.state.write().unwrap();
    write_lock_state.move_one_class(
      delta.src_day,
      delta.src_timeslot,
      delta.dst_day,
      delta.dst_timeslot,
      delta.class_key,
    );
  }

  fn eval_cost(&mut self) -> f64 {
    self.cost_counter.store(0, Ordering::SeqCst);
    self.start_eval_barrier.wait();
    // All threads start to eval
    // All threads add to the cost_counter
    self.finish_eval_barrier.wait();
    // Wait for all threads to finish
    // Return the sum from all threads
    let r = self.cost_counter.load(Ordering::SeqCst);
    let r = r as f64;

    r / (EVALUATORS_FACTOR as f64)
  }

  fn get_curr_state(&self) -> ClassCalendar {
    self.state.read().unwrap().clone()
  }
}

pub(crate) fn assign_classrooms(
  state: &ClassCalendar,
  constraints: &OptimizationConstraints,
) -> BTreeMap<ClassroomAssignmentKey, Classroom> {
  let matching = assign_classrooms_matching(state, constraints);
  matching.collect()
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
enum ClassroomAssignmentVertex {
  Class(ClassKey),
  Classroom(Classroom),
}

fn timeslot_assign_classrooms<'a>(
  state: &'a ClassCalendar,
  constraints: &'a OptimizationConstraints,
  day: week_calendar::Day,
  timeslot: week_calendar::Timeslot,
  available_classrooms: Rc<[Vec<Classroom>; ClassroomType::VARIANTS.len()]>,
) -> impl Iterator<Item = (ClassroomAssignmentKey, Classroom)> + 'a {
  let mut edges: Vec<(ClassroomAssignmentVertex, ClassroomAssignmentVertex)> = Vec::new();
  for class_key in state
    .iter_class_keys()
    .filter(|k| state.get_count(day, timeslot, *k) > 0)
  {
    let class = constraints.classes.get(class_key).unwrap();
    let classrooms = class
      .allowed_classroom_types
      .iter()
      .map(|classroom_type| {
        ClassroomType::VARIANTS
          .iter()
          .position(|v| *v == classroom_type)
          .unwrap()
      })
      .flat_map(|classroom_type_i| &available_classrooms[classroom_type_i])
      .unique();
    for classroom in classrooms {
      edges.push((
        ClassroomAssignmentVertex::Class(class_key),
        ClassroomAssignmentVertex::Classroom(*classroom),
      ));
    }
  }
  let matching = hopcroft_karp::matching(&edges);
  matching.into_iter().map(move |(a, b)| match (a, b) {
    (
      ClassroomAssignmentVertex::Class(class_key),
      ClassroomAssignmentVertex::Classroom(classroom),
    ) => (
      ClassroomAssignmentKey {
        day,
        timeslot,
        class_key,
      },
      classroom,
    ),
    _ => unreachable!(),
  })
}

fn iter_week() -> impl Iterator<Item = (week_calendar::Day, week_calendar::Timeslot)> {
  week_calendar::Day::all().flat_map(|d| week_calendar::Timeslot::all().map(move |t| (d, t)))
}

fn assign_classrooms_matching<'a>(
  state: &'a ClassCalendar,
  constraints: &'a OptimizationConstraints,
) -> impl Iterator<Item = (ClassroomAssignmentKey, Classroom)> + 'a {
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
  let available_classrooms = Rc::new(available_classrooms);

  iter_week().flat_map(move |(day, timeslot)| {
    timeslot_assign_classrooms(
      state,
      constraints,
      day,
      timeslot,
      available_classrooms.clone(),
    )
  })
}

fn count_classroom_assignment_collisions(
  state: &ClassCalendar,
  constraints: &OptimizationConstraints,
) -> usize {
  state
    .get_entries()
    .len()
    .checked_sub(assign_classrooms_matching(state, constraints).count())
    .expect("Can't be more matching than class entries")
}

use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub enum ProgressOption {
  ProgressBar(indicatif::ProgressBar),
  MultiProgress(indicatif::MultiProgress),
  #[default]
  None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TemperatureFunction {
  Linear,
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
pub struct AdvancedSimulationOptions {
  pub progress_bar_update_interval: usize,
  #[serde(skip)]
  pub live_update: Option<LiveUpdate>,
}

impl Default for AdvancedSimulationOptions {
  fn default() -> Self {
    Self {
      progress_bar_update_interval: 100,
      live_update: None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum StopCondition {
  Steps(usize),
  Time(Duration),
}

impl Default for StopCondition {
  fn default() -> Self {
    StopCondition::Steps(0)
  }
}

#[derive(Debug, Clone)]
pub struct LiveUpdate {
  pub channel: std::sync::mpsc::Sender<ClassCalendar>,
  pub live_update_interval: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimulationOptions {
  pub stop_condition: StopCondition,
  pub initial_state: ClassCalendar,
  #[serde(skip)]
  pub progress: ProgressOption,
  pub temperature_function: TemperatureFunction,
  pub advanced_options: AdvancedSimulationOptions,
}

#[derive(Default, Debug)]
pub struct SimulatedAnnealingOptimizer {}

impl ClassCalendarOptimizer for SimulatedAnnealingOptimizer {
  type OptimizerOptions = SimulationOptions;

  fn generate_class_calendar(
    &mut self,
    constraints: crate::OptimizationConstraints,
    options: Self::OptimizerOptions,
    cost_function: Option<crate::CostFunction>,
  ) -> crate::ClassCalendar {
    let mut rng = thread_rng();
    let result = simulated_annealing(&constraints, options, &mut rng);
    result.final_calendar
  }
}
