pub mod calendar_widget;
pub mod calendars;
pub mod evaluators;
pub mod perturbators;
pub mod real_counter;
pub mod simulation;
pub mod timeslot;
pub mod app;

use std::f32::consts::PI;

use rand::thread_rng;

use crate::simulation::Simulation;
use crate::app::MyApp;

fn temperature(x: f32) -> f32 {
    (1.0 / (x + 1.0)) - 0.5 * ((2.0 * PI * x).cos()).powi(2)
}

fn main() {
    let class_hours: Vec<(usize, usize)> = vec![(0, 4), (1, 6), (2, 3), (4, 6), (5, 6), (7, 2)];
    let mut rng = thread_rng();
    let mut simulation = Simulation::new_no_init(class_hours);
    simulation.init_random_state(&mut rng);
    simulation.update_current_energy();

    let original_state = simulation.get_current_state().clone();

    // let total_steps = 100000;
    // clearscreen::clear().unwrap();
    // println!("{}", simulation.get_current_state());
    // for step in 0..total_steps {
    //    simulation.step(temperature(step as f32 / total_steps as f32), &mut rng);
    // }
    // clearscreen::clear().unwrap();
    // println!("{}", original_state);
    // println!("{}", simulation.get_current_state());

    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(MyApp::new(
        simulation,
        rng
    )), options);
}
