pub mod app;
pub mod class_editor;
pub mod professor_editor;
pub mod professor_schedule_widget;
pub mod school_schedule;
pub mod simple_schedule_widget;
pub mod simulation;
pub mod timeslot;
pub mod week_calendar;
pub mod database_importer;

use crate::app::MyApp;

fn main() {
  let schedule = database_importer::import_database().expect("Failed to import");
  let options = eframe::NativeOptions::default();
  eframe::run_native("my_app", options, Box::new(|cc| Box::new({
    let mut app = MyApp::new(cc);
    app.school_schedule = schedule;
    app
  })))
    .expect("Something went wrong!");
}
