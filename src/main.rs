pub mod app;
pub mod class_editor;
pub mod professor_editor;
pub mod professor_schedule_widget;
pub mod school_schedule;
pub mod simple_schedule_widget;
pub mod timeslot;
pub mod week_calendar;
pub mod simulation;

use crate::app::MyApp;

fn main() {
  let options = eframe::NativeOptions::default();
  eframe::run_native("my_app", options, Box::new(|cc| Box::new(MyApp::new(cc))))
    .expect("Something went wrong!");
}
