use std::thread::{self, JoinHandle};

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

pub(crate) fn generate_schedule(constraints: SimulationConstraints) -> JoinHandle<ClassCalendar> {
  thread::spawn(move || simulated_annealing(constraints, 100_000))
}

#[derive(Default, Serialize, Deserialize)]
struct Stats {
  accepted: Vec<bool>,
  acceptance_probability: Vec<f64>,
  temperature: Vec<f64>,
  curr_cost: Vec<f64>,
  new_cost: Vec<f64>,
  x: Vec<f64>,
}

impl Stats {
  fn with_capacity(capacity: usize) -> Self {
    Stats {
      accepted: Vec::with_capacity(capacity),
      acceptance_probability: Vec::with_capacity(capacity),
      temperature: Vec::with_capacity(capacity),
      new_cost: Vec::with_capacity(capacity),
      curr_cost: Vec::with_capacity(capacity),
      x: Vec::with_capacity(capacity),
    }
  }
}

fn simulated_annealing(constraints: SimulationConstraints, steps: u32) -> ClassCalendar {
  // let seed: [u8; 32] = "Aritz123Aritz123Aritz123Aritz123"
  //   .as_bytes()
  //   .try_into()
  //   .unwrap();
  // let mut rng = rand::rngs::StdRng::from_seed(seed);
  let mut rng = rand::rngs::ThreadRng::default();

  let mut stats = Stats::with_capacity(steps as usize);

  let mut state = random_init(&constraints, &mut rng);
  let mut state_cost = cost(&state, &constraints);

  for step in 0..steps {
    stats.curr_cost.push(state_cost);
    let t = {
      let x = ((step + 1) as f64) / (steps as f64); stats.x.push(x); 
      let t = temperature(x); stats.temperature.push(t);
      t
    };
    let old_cost = state_cost;
    let delta = state.move_one_class_random(&mut rng);
    
    let new_cost = cost(&state, &constraints); stats.new_cost.push(new_cost);

    let ap = acceptance_probability(old_cost, new_cost, t); stats.acceptance_probability.push(ap);
    if ap >= rng.gen_range(0.0..=1.0) {
      stats.accepted.push(true);
      // keep change
      state_cost = new_cost;
    } else {
      stats.accepted.push(false);
      revert_change(&mut state, &delta);
      state_cost = old_cost;
    }

    if step % 10_000 == 0 {
      println!("Step: {}/{}", step, steps);
    }
  }

  info!("Saving stats.");
  let file = std::fs::File::create("stats.json").unwrap();
  let writer = std::io::BufWriter::new(file);
  serde_json::ser::to_writer(writer, &stats).unwrap();

  state
}

fn acceptance_probability(old_cost: f64, new_cost: f64, temperature: f64) -> f64 {
  if new_cost < old_cost {
    1.0
  } else {
    (-(new_cost - old_cost) / temperature.max(f64::EPSILON)).exp()
  }
}

fn temperature(x: f64) -> f64 {
  // 10.0 - 10.0 * x
  //0.0
  // 7.5*(0.5*(5.0*std::f64::consts::PI*x+std::f64::consts::FRAC_2_PI).sin()+0.5)
  if x <= 0.9 { 7.5*(0.5*(1.1*7.0*std::f64::consts::PI*x+std::f64::consts::FRAC_2_PI).sin()+0.5) } else { 0.0 }
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
    delta.dst_timeslot_idx,
    delta.class_id,
  );
}

fn cost(state: &ClassCalendar, constraints: &SimulationConstraints) -> f64 {
  10.0 * heuristics::same_timeslot_classes_count(state, constraints)
    + 7.0 * heuristics::count_not_available(state, constraints)
    + 2.0 * heuristics::count_available_if_needed(state, constraints)
}
