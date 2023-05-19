pub mod app;
pub mod class_editor;
pub mod database_importer;
pub mod professor_editor;
pub mod professor_schedule_widget;
pub mod school_schedule;
pub mod simple_schedule_widget;
pub mod simulation;
pub mod timeslot;
pub mod week_calendar;

use crate::app::MyApp;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() {
  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

  database_importer::import_temporary_database().expect("Error");
  let schedule = database_importer::parse_database_data().expect("Failed to import");
  let options = eframe::NativeOptions::default();
  eframe::run_native(
    "my_app",
    options,
    Box::new(|cc| {
      Box::new({
        let mut app = MyApp::new(cc);
        app.school_schedule = schedule;
        app
      })
    }),
  )
  .expect("Something went wrong!");
}
