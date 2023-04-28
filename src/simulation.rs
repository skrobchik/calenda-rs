use std::thread::{JoinHandle, self};

use rand::{rngs::ThreadRng, Rng};

use crate::{week_calendar::WeekCalendar, school_schedule::{Classes, SimulationConstraints}};

pub fn generate_schedule(constraints: SimulationConstraints) -> JoinHandle<WeekCalendar<Classes>> {
    thread::spawn(move || {
        simulated_annealing(constraints, 1_000_000)
    })
}

fn simulated_annealing(constraints: SimulationConstraints, steps: u32) -> WeekCalendar<Classes> {
    let mut rng: ThreadRng = Default::default();
    
    let mut state = random_init(&constraints, &mut rng);

    for step in 0..steps {
        let t = temperature(1.0 - ((step+1) as f32)/100.0);
        let new_state = random_neighbour(state.clone(), &mut rng);
        let old_cost = cost(&state, &constraints);
        let new_cost = cost(&new_state, &constraints);
        if acceptance_probability(old_cost, new_cost, t) >= rng.gen_range(0.0..=1.0) {
            state = new_state;
        }
    }

    state
}

fn acceptance_probability(old_cost: f32, new_cost: f32, temperature: f32) -> f32 {
    todo!()
}

fn temperature(x: f32) -> f32 {
    todo!()
}

fn random_init(constraints: &SimulationConstraints, rng: &mut ThreadRng) -> WeekCalendar<Classes> {
    todo!()
}

fn random_neighbour(state: WeekCalendar<Classes>, rnd: &mut ThreadRng) -> WeekCalendar<Classes> {
    todo!()
}

fn cost(state: &WeekCalendar<Classes>, constraints: &SimulationConstraints) -> f32 {
    todo!()
}