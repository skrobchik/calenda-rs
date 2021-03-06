pub mod app;
pub mod calendar_widget;
pub mod calendars;
pub mod evaluators;
pub mod metadata_register;
pub mod real_counter;
pub mod thread_simulation;
pub mod timeslot;

use crate::app::MyApp;

fn main() {
  let options = eframe::NativeOptions::default();
  eframe::run_native(
    "my_app",
    options,
    Box::new(|cc| Box::new(MyApp::new(cc))),
  );
}
