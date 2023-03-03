pub mod app;
pub mod calendars;
pub mod evaluators;
pub mod metadata_register;
pub mod real_counter;
pub mod register;
pub mod school_schedule;
pub mod simple_calendar_widget;
pub mod simple_schedule_widget;
pub mod thread_simulation;
pub mod timeslot;
pub mod timeslots;
pub mod week_calendar;

use crate::app::MyApp;

fn main() {
  let options = eframe::NativeOptions::default();
  eframe::run_native("my_app", options, Box::new(|cc| Box::new(MyApp::new(cc))));
}
