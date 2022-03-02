use crate::evaluators;
use crate::evaluators::Evaluator;
use crate::timeslot::{self, DayTimeSlot, DAY_RANGE, TIMESLOT_RANGE};

use rand::prelude::{IteratorRandom, StdRng};
use rand::{Rng, SeedableRng};

use crate::calendars::{CalendarState, ClassId};

/// Returns x-1 or x+1 randomly as long as the result is withing the range's bounds. If x-1 or x+1 are not inside the range, returns x.
fn random_shift_bounded(x: usize, range: std::ops::Range<usize>, rng: &mut StdRng) -> usize {
    let smaller_available = range.contains(&(x - 1));
    let larger_available = range.contains(&(x + 1));
    if !smaller_available && !larger_available {
        return x;
    }
    if !smaller_available && larger_available {
        return x + 1;
    }
    if smaller_available && !larger_available {
        return x - 1;
    }
    match rng.gen_bool(0.5) {
        true => x - 1,
        false => x + 1,
    }
}

pub struct Simulation {
    current_state: CalendarState,
    current_energy: f32,
    simulation_step_count: usize,
    temperature_function: Box<dyn Fn(usize) -> f32 + Send + Sync>,
    acceptance_probability_function: Box<dyn Fn(f32, f32, f32) -> f32 + Send + Sync>,
    rng: StdRng,
}

impl Simulation {
    pub fn from_entropy() -> Self {
        Self {
            current_state: Default::default(),
            current_energy: Default::default(),
            simulation_step_count: Default::default(),
            temperature_function: Box::new(default_temperature_function),
            acceptance_probability_function: Box::new(default_acceptance_probability_function),
            rng: StdRng::from_entropy(),
        }
    }
    pub fn add_class_hours(&mut self, class_hours: &Vec<(ClassId, usize)>) {
        for (class_id, count) in class_hours {
            for _ in 0..*count {
                self.current_state
                    .add_session(*class_id, DayTimeSlot::random(&mut self.rng))
            }
        }
        self.current_energy = self.calculate_energy(&self.current_state);
    }
    fn calculate_energy(&self, state: &CalendarState) -> f32 {
        let e1 = evaluators::Colliding::new(5.0);
        let e2 = evaluators::Daylight::new(timeslot::TIMESLOT_10_00, timeslot::TIMESLOT_18_00, 2.0);
        let e3 = evaluators::GapCount::new(2.0);
        let e4 = evaluators::DailyWorkDifference::new(2.0);
        e1.evaluate(&state) + e2.evaluate(&state) + e3.evaluate(&state) + e4.evaluate(&state)
    }
    fn get_random_neighbor(&mut self) -> Option<CalendarState> {
        let session = self
            .current_state
            .get_session_set()
            .keys()
            .choose(&mut self.rng)?;
        let source_daytime = session.t.clone();
        let target_daytime = match self.rng.gen_range(0u8..=1) {
            0 => DayTimeSlot {
                day: source_daytime.day,
                timeslot: random_shift_bounded(
                    source_daytime.timeslot,
                    TIMESLOT_RANGE,
                    &mut self.rng,
                ),
            },
            1 => DayTimeSlot {
                day: random_shift_bounded(source_daytime.day, DAY_RANGE, &mut self.rng),
                timeslot: TIMESLOT_RANGE.choose(&mut self.rng).unwrap(),
            },
            _ => unreachable!(),
        };
        let mut new_state = self.current_state.clone();
        new_state
            .move_session(session.class_id, source_daytime, target_daytime)
            .expect("Something went horribly wrong");
        Some(new_state)
    }
    pub fn step(&mut self) {
        let new_state = self.get_random_neighbor().unwrap();
        let new_energy = self.calculate_energy(&new_state);
        let temperature = (self.temperature_function)(self.simulation_step_count);
        let p =
            (self.acceptance_probability_function)(self.current_energy, new_energy, temperature);
        if self.rng.gen_bool(p.into()) {
            self.current_state = new_state;
            self.current_energy = new_energy;
        }
        self.simulation_step_count += 1;
    }
    pub fn get_current_state(&self) -> &CalendarState {
        &self.current_state
    }
    pub fn get_current_energy(&self) -> f32 {
        self.current_energy
    }
}

fn default_temperature_function(x: usize) -> f32 {
    let x = x as f32 / 1000000.0;
    (1.0 / (x + 1.0)) - 0.5 * ((2.0 * std::f32::consts::PI * x).cos()).powi(2)
}

fn default_acceptance_probability_function(
    current_energy: f32,
    new_energy: f32,
    temperature: f32,
) -> f32 {
    if new_energy < current_energy {
        1.0
    } else {
        (-(new_energy - current_energy) / temperature).exp()
    }
}
