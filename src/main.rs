use std::f32::consts::PI;

use calendars::CalendarState;
use rand::{thread_rng, Rng};
use timeslot::TIMESLOT_18_00;

use crate::{evaluators::Evaluator, perturbators::Perturbator};

pub mod calendars;
pub mod evaluators;
pub mod perturbators;
pub mod real_counter;
pub mod timeslot;

fn temperature(x: f32) -> f32 {
    (1.0 / (x + 1.0)) - 0.5 * ((2.0 * PI * x).cos()).powi(2)
}

fn acceptance_probability(e: f32, e_new: f32, temp: f32) -> f32 {
    if e_new < e {
        1.0
    } else {
        (-(e_new - e) / temp).exp()
    }
}

fn main() {
    let course_hours: Vec<(usize, usize)> = vec![(0, 4), (1, 6), (2, 3), (4, 6), (5, 6), (7, 2)];
    let mut rng = thread_rng();

    let e1 = evaluators::Colliding::new(5.0);
    let e2 = evaluators::Daylight::new(timeslot::TIMESLOT_10_00, TIMESLOT_18_00, 2.0);
    let e3 = evaluators::GapCount::new(2.0);
    let e4 = evaluators::DailyWorkDifference::new(2.0);

    let p1 = perturbators::MoveHour::new();
    let p2 = perturbators::MoveDay::new();

    let mut state = CalendarState::random_new(&course_hours, &mut rng);
    let mut energy =
        e1.evaluate(&state) + e2.evaluate(&state) + e3.evaluate(&state) + e4.evaluate(&state);

    let original_state = state.clone();

    let step_count = 100000;
    clearscreen::clear().unwrap();
    println!("{}", state);
    for step in 0..step_count {
        let temp = temperature(1.0 - (step as f32 + 1.0) / step_count as f32);
        let mut new_state = state.clone();
        match step % 2 {
            0 => p1.perturbate(&mut new_state, &mut rng),
            1 => p2.perturbate(&mut new_state, &mut rng),
            _ => unreachable!(),
        }
        let new_energy = e1.evaluate(&new_state)
            + e2.evaluate(&new_state)
            + e3.evaluate(&new_state)
            + e4.evaluate(&new_state);
        let p = acceptance_probability(energy, new_energy, temp);
        if p >= rng.gen_range(0.0..=1.0) {
            state = new_state;
            energy = new_energy;
        }
        if step % 100 == 0 {
            clearscreen::clear().unwrap();
            println!("{}", state);
            println!("{}/{}", step, step_count);
            println!("energy: {}", energy);
        }
    }
    clearscreen::clear().unwrap();
    println!("{}", original_state);
    println!("{}", state);
}
