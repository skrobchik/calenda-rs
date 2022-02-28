use crate::evaluators::Evaluator;
use crate::perturbators::Perturbator;
use crate::timeslot;
use crate::{evaluators, perturbators};
use rand::prelude::ThreadRng;
use rand::Rng;

use crate::calendars::{CalendarState, ClassId};

pub struct Simulation {
    class_hours: Vec<(ClassId, usize)>,
    current_state: CalendarState,
    current_energy: f32,
    simulation_step_count: usize,
}

impl Simulation {
    pub fn new_no_init(class_hours: Vec<(ClassId, usize)>) -> Self {
        Simulation {
            class_hours,
            current_state: CalendarState::new(),
            current_energy: 0.0,
            simulation_step_count: 0,
        }
    }
    pub fn init_random_state(&mut self, rng: &mut ThreadRng) {
        self.current_state = CalendarState::random_new(&self.class_hours, rng);
    }
    pub fn update_current_energy(&mut self) {
        self.current_energy = self.calculate_energy(&self.current_state);
    }
    fn acceptance_probability(&self, temperature: f32, new_energy: f32) -> f32 {
        if new_energy < self.current_energy {
            1.0
        } else {
            (-(new_energy - self.current_energy) / temperature).exp()
        }
    }

    fn calculate_energy(&self, state: &CalendarState) -> f32 {
        let e1 = evaluators::Colliding::new(5.0);
        let e2 = evaluators::Daylight::new(timeslot::TIMESLOT_10_00, timeslot::TIMESLOT_18_00, 2.0);
        let e3 = evaluators::GapCount::new(2.0);
        let e4 = evaluators::DailyWorkDifference::new(2.0);
        e1.evaluate(&state) + e2.evaluate(&state) + e3.evaluate(&state) + e4.evaluate(&state)
    }
    pub fn step(&mut self, temperature: f32, rng: &mut ThreadRng) {
        let p1 = perturbators::MoveHour::new();
        let p2 = perturbators::MoveDay::new();
        let mut new_state = self.current_state.clone();
        match self.simulation_step_count % 2 {
            0 => p1.perturbate(&mut new_state, rng),
            1 => p2.perturbate(&mut new_state, rng),
            _ => unreachable!(),
        }
        let new_energy = self.calculate_energy(&new_state);
        let p = self.acceptance_probability(temperature, new_energy);
        if p >= rng.gen_range(0.0..=1.0) {
            self.current_state = new_state;
            self.current_energy = new_energy;
        }
        self.simulation_step_count += 1;
    }
    pub fn get_current_energy(&self) -> f32 {
        self.current_energy
    }
    pub fn get_current_state(&self) -> &CalendarState {
        &self.current_state
    }
    pub fn get_step_count(&self) -> usize {
        self.simulation_step_count
    }
}
