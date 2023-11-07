pub mod app;
pub mod class_editor;
pub mod class_filter;
pub mod database_importer;
pub mod heuristics;
pub mod professor_editor;
pub mod professor_schedule_widget;
pub mod school_schedule;
pub mod simple_schedule_widget;
pub mod simulation;
pub mod timeslot;
pub mod week_calendar;

use std::sync::{Mutex, Arc};

use crate::app::MyApp;

use indicatif::MultiProgress;
use simulation::ScheduleGenerationOptions;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub(crate) fn load_results<P: AsRef<std::path::Path>>(path: P) -> Vec<simulation::ScheduleGenerationOutput> {
  let path = path.as_ref();
  let file = std::fs::File::open(path).unwrap();
  let reader = std::io::BufReader::new(file);
  serde_json::from_reader(reader).unwrap()
}

fn run_app() {
  database_importer::import_temporary_database().expect("Error");
  let mut schedule = database_importer::parse_database_data().expect("Failed to import");
  
  let simulation_output = load_results("results3.json").into_iter().nth(20).unwrap();
  println!("Num Steps: {}", simulation_output.best_schedule_run_report.num_steps);
  println!("Cost: {}", simulation_output.best_schedule_cost);
  let class_calendar = simulation_output.best_schedule;
  schedule.replace_class_calendar(class_calendar).unwrap();


  let options = eframe::NativeOptions::default();
  eframe::run_native(
    "my_app",
    options,
    Box::new(|cc| {
      Box::new({
        let mut app = MyApp::new(cc);
        app.school_schedule = schedule; // Overwrite saved SchoolSchedule
        app
      })
    }),
  )
  .expect("Something went wrong!");
}

fn run_experiment_1() {
  database_importer::import_temporary_database().expect("Error");
  let schedule = database_importer::parse_database_data().expect("Failed to import");

  let steps_vec = vec![
    128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728
  ];

  let mp = Arc::new(Mutex::new(MultiProgress::new()));
  let handles: Vec<std::thread::JoinHandle<simulation::ScheduleGenerationOutput>> = steps_vec
    .into_iter()
    .map(|steps| {
      println!("Starting run for {} steps", steps);
      let result = simulation::generate_schedule(ScheduleGenerationOptions {
        simulation_constraints: schedule.get_simulation_constraints().clone(),
        steps,
        parallel_count: 1,
        initial_state: None,
        multi_progress: Some(mp.clone()),
      });
      result
    })
    .collect();

  let results: Vec<simulation::ScheduleGenerationOutput> = handles
    .into_iter()
    .map(|handle| handle.join().unwrap())
    .collect();

  let file = std::fs::File::create("results.json").unwrap();
  let writer = std::io::BufWriter::new(file);
  serde_json::ser::to_writer(writer, &results).unwrap()
}

fn main() {
  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

  // run_experiment_1()
  run_app()
}
