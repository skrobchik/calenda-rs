use std::thread::{self, JoinHandle};

use rand::prelude::*;

use crate::{
  heuristics,
  school_schedule::{
    class_calendar::{ClassCalendar, ClassEntryDelta},
    SimulationConstraints,
  },
  timeslot,
};

pub(crate) fn generate_schedule(constraints: SimulationConstraints) -> JoinHandle<ClassCalendar> {
  thread::spawn(move || simulated_annealing(constraints, 1_000_000))
}

fn simulated_annealing(constraints: SimulationConstraints, steps: u32) -> ClassCalendar {
  let seed: [u8; 32] = "Aritz123Aritz123Aritz123Aritz123"
    .as_bytes()
    .try_into()
    .unwrap();
  let mut rng = rand::rngs::StdRng::from_seed(seed);

  let mut state = random_init(&constraints, &mut rng);
  let mut state_cost = cost(&state, &constraints);

  for step in 0..steps {
    let t = temperature(1.0 - ((step + 1) as f32) / 100.0);
    let old_cost = state_cost;

    let delta = state.move_one_class_random(&mut rng);

    let new_cost = cost(&state, &constraints);

    if acceptance_probability(old_cost, new_cost, t) >= rng.gen_range(0.0..=1.0) {
      // keep change
      state_cost = new_cost;
    } else {
      revert_change(&mut state, &delta);
      state_cost = old_cost;
    }

    if step % 10_000 == 0 {
      println!("Step: {}/{}", step, steps);
    }
  }

  state
}

fn acceptance_probability(old_cost: f32, new_cost: f32, temperature: f32) -> f32 {
  if new_cost < old_cost {
    1.0
  } else {
    (-(new_cost - old_cost) / temperature.max(f32::EPSILON)).exp()
  }
}

fn temperature(x: f32) -> f32 {
  100.0 * 0.95_f32.powf(x)
}

fn random_init(constraints: &SimulationConstraints, rng: &mut StdRng) -> ClassCalendar {
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

fn cost(state: &ClassCalendar, constraints: &SimulationConstraints) -> f32 {
  1.0 * heuristics::same_timeslot_classes_count(state, constraints)
    + 7.0 * heuristics::count_not_available(state, constraints)
    + 2.0 * heuristics::count_available_if_needed(state, constraints)
}
