pub mod app;
pub mod calendar_widget;
pub mod calendars;
pub mod evaluators;
pub mod metadata_register;
pub mod real_counter;
pub mod thread_simulation;
pub mod timeslot;

use thread_simulation::ThreadSimulation;

use crate::app::MyApp;

fn main() {
    let thread_simulation = ThreadSimulation::new();

    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(MyApp::new(thread_simulation)), options);
}
