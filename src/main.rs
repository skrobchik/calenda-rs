pub mod calendar_widget;
pub mod calendars;
pub mod evaluators;
pub mod real_counter;
pub mod simulation;
pub mod thread_simulation;
pub mod timeslot;
pub mod app;
pub mod metadata_register;

use simulation::Simulation;
use thread_simulation::ThreadSimulation;

use crate::app::MyApp;

fn main() {
    let class_hours: Vec<(usize, usize)> = vec![(0, 4), (1, 6), (2, 3), (4, 6), (5, 6), (7, 2)];
    let mut simulation = Simulation::from_entropy();
    simulation.add_class_hours(&class_hours);
    let thread_simulation = ThreadSimulation::new(simulation);

    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(MyApp::new(
        thread_simulation,
    )), options);
}
