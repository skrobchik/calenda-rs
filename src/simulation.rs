use std::thread::{self, JoinHandle};

use tracing::{debug};

use rand::{rngs::ThreadRng, Rng};

use crate::{
  school_schedule::{Classes, SimulationConstraints, MAX_CLASSES},
  timeslot,
  week_calendar::WeekCalendar,
};

pub fn generate_schedule(constraints: SimulationConstraints) -> JoinHandle<WeekCalendar<Classes>> {
  thread::spawn(move || simulated_annealing(constraints, 100_000))
}

fn simulated_annealing(constraints: SimulationConstraints, steps: u32) -> WeekCalendar<Classes> {
  let mut rng: ThreadRng = Default::default();

  let mut state = random_init(&constraints, &mut rng);
  let mut state_cost = cost(&state, &constraints);

  let mut no_change_count = 0;

  for step in 0..steps {
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
  }

  debug!("No Changes: {}", no_change_count);

  state
}

fn acceptance_probability(old_cost: f32, new_cost: f32, temperature: f32) -> f32 {
  if new_cost < old_cost {
    1.0
  } else {
    (-(new_cost-old_cost)/temperature.max(f32::EPSILON)).exp()
  }
}

fn temperature(x: f32) -> f32 {
  100.0 * 0.95_f32.powf(x)
}

fn random_init(constraints: &SimulationConstraints, rng: &mut ThreadRng) -> WeekCalendar<Classes> {
  let mut state: WeekCalendar<Classes> = Default::default();

  for (class_id, class) in constraints.classes.iter().enumerate() {
    if let Some(class) = class {
      for _ in 0..class.class_hours {
        let rand_timeslot = rng.gen_range(timeslot::TIMESLOT_RANGE);
        let rand_day = rng.gen_range(timeslot::DAY_RANGE);
        state.get_mut(rand_day, rand_timeslot).unwrap()[class_id] += 1;
      }
    }
  }

  state
}

struct Delta {
  i1: usize,
  i2: usize,
  class_id: usize
}

fn random_change(state: &mut WeekCalendar<Classes>, rnd: &mut ThreadRng) -> Option<Delta> {
  let n = state.data.len();
  let i1 = rnd.gen_range(0..n);
  let i2 = rnd.gen_range(0..n);
  let class_id = rnd.gen_range(0..MAX_CLASSES);

  if state.data[i1].data[class_id] > 0 {
    state.data[i1].data[class_id] -= 1;
    state.data[i2].data[class_id] += 1;
    debug!("Class Id: {}, {} -> {}", class_id, i1, i2);
    Some(Delta {
      i1, i2, class_id
    })
  } else { None }
}

fn revert_change(state: &mut WeekCalendar<Classes>, delta: &Delta) {
  let i1 = delta.i1; let i2: usize = delta.i2; let class_id = delta.class_id;
  state.data[i1].data[class_id] += 1;
  state.data[i2].data[class_id] -= 1;
  debug!("Class Id: {}, {} <- {}", class_id, i1, i2);
}

fn cost(state: &WeekCalendar<Classes>, constraints: &SimulationConstraints) -> f32 {
  let mut cost: f32 = 0.0;

  let mut same_timeslot_classes_count: f32 = 0.0;
  for classes in state.data.iter() {
    let same_timeslot: bool = classes.data.iter().filter(|x| **x > 1).nth(1).is_some();
    if same_timeslot {
      same_timeslot_classes_count += 1.0;
    }
  }

  cost += 1.0 * same_timeslot_classes_count;


  cost
}
