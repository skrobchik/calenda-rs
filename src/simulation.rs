use std::thread::{self, JoinHandle};

use tracing::debug;

use rand::prelude::*;

use crate::{
  heuristics,
  school_schedule::{SimulationConstraints, TimeslotClassHours},
  timeslot,
  week_calendar::WeekCalendar,
};

pub(crate) fn generate_schedule(
  constraints: SimulationConstraints,
) -> JoinHandle<WeekCalendar<TimeslotClassHours>> {
  thread::spawn(move || simulated_annealing(constraints, 1_000_000))
}

fn count_total_classes(state: &WeekCalendar<TimeslotClassHours>) -> u32 {
  let mut total_count: u32 = 0;
  for day in timeslot::DAY_RANGE {
    for timeslot in timeslot::TIMESLOT_RANGE {
      total_count += state.get(day, timeslot).unwrap().iter().map(|x| *x as u32).sum::<u32>();
    }
  }
  total_count
}

fn simulated_annealing(
  constraints: SimulationConstraints,
  steps: u32,
) -> WeekCalendar<TimeslotClassHours> {
  let seed: [u8; 32] = "Aritz123Aritz123Aritz123Aritz123".as_bytes().try_into().unwrap();
  let mut rng = rand::rngs::StdRng::from_seed(seed);


  let mut state = random_init(&constraints, &mut rng);
  let mut state_cost = cost(&state, &constraints);

  let mut no_change_count = 0;

  let original_total_count = count_total_classes(&state);
  let mut violations_count = 0;

  for step in 0..steps {

    let current_total_count = count_total_classes(&state);
    if current_total_count != original_total_count {
      violations_count += 1;
    }

    let t = temperature(1.0 - ((step + 1) as f32) / 100.0);
    let old_cost = state_cost;
    let delta = random_change(&mut state, &mut rng);
    let new_cost = cost(&state, &constraints);
    if let Some(delta) = delta {
      if acceptance_probability(old_cost, new_cost, t) >= rng.gen_range(0.0..=1.0) {
        // keep change
        state_cost = new_cost;
      } else {
        revert_change(&mut state, &delta);
        state_cost = old_cost;
      }
    } else {
      no_change_count += 1;
    }
    if step % 10_000 == 0 {
      println!("Step: {}/{}", step, steps);
    }
  }

  debug!("No Changes: {}", no_change_count);

  println!("Num Violations: {}", violations_count);

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

fn random_init(
  constraints: &SimulationConstraints,
  rng: &mut StdRng,
) -> WeekCalendar<TimeslotClassHours> {
  let mut state: WeekCalendar<TimeslotClassHours> = Default::default();

  for (class_id, class) in constraints.classes.iter().enumerate() {
    for _ in 0..class.class_hours {
      let rand_timeslot = rng.gen_range(timeslot::TIMESLOT_RANGE);
      let rand_day = rng.gen_range(timeslot::DAY_RANGE);
      state.get_mut(rand_day, rand_timeslot).unwrap()[class_id] += 1;
    }
  }

  state
}

struct Delta {
  i1: usize,
  i2: usize,
  class_id: usize,
}

fn random_change(
  state: &mut WeekCalendar<TimeslotClassHours>,
  rnd: &mut StdRng,
) -> Option<Delta> {
  let n = state.data.len();
  let i1 = rnd.gen_range(0..n);
  let i2 = rnd.gen_range(0..n);
  let next_class_id = state.data[i1].len().max(state.data[i2].len());
  if next_class_id == 0 {
    return None;
  }
  let class_id = rnd.gen_range(0..next_class_id);

  if state.data[i1][class_id] > 0 {
    state.data[i1][class_id] -= 1;
    state.data[i2][class_id] += 1;
    debug!("Class Id: {}, {} -> {}", class_id, i1, i2);
    Some(Delta { i1, i2, class_id })
  } else {
    None
  }
}

fn revert_change(state: &mut WeekCalendar<TimeslotClassHours>, delta: &Delta) {
  let i1 = delta.i1;
  let i2: usize = delta.i2;
  let class_id = delta.class_id;
  state.data[i1][class_id] += 1;
  state.data[i2][class_id] -= 1;
  debug!("Class Id: {}, {} <- {}", class_id, i1, i2);
}

fn cost(state: &WeekCalendar<TimeslotClassHours>, constraints: &SimulationConstraints) -> f32 {
  1.0 * heuristics::same_timeslot_classes_count(state, constraints)
    + 7.0 * heuristics::count_not_available(state, constraints)
    + 2.0 * heuristics::count_available_if_needed(state, constraints)
}
