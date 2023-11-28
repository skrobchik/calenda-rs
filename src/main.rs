pub mod app;
pub mod class_editor;
pub mod class_filter;
pub mod color_list;
pub mod database_importer;
pub mod heuristics;
pub mod professor_editor;
pub mod professor_schedule_widget;
pub mod school_schedule;
pub mod simple_schedule_widget;
pub mod simulation;
pub mod stats_tracker;
pub mod timeslot;
pub mod week_calendar;

use crate::app::MyApp;

use indicatif::MultiProgress;
use simulation::{SimulationOptions, TemperatureFunction};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub(crate) fn load_results<P: AsRef<std::path::Path>>(
  path: P,
) -> Vec<simulation::SimulationOutput> {
  let path = path.as_ref();
  let file = std::fs::File::open(path).unwrap();
  let reader = std::io::BufReader::new(file);
  serde_json::from_reader(reader).unwrap()
}

#[allow(dead_code)]
fn run_app() {
  database_importer::import_temporary_database().expect("Error");
  let schedule = database_importer::parse_database_data().expect("Failed to import");

  // let simulation_output = load_results("results3.json").into_iter().nth(20).unwrap();
  // println!("Num Steps: {}", simulation_output.best_schedule_run_report.num_steps);
  // println!("Cost: {}", simulation_output.best_schedule_cost);
  // let class_calendar = simulation_output.best_schedule;
  // schedule.replace_class_calendar(class_calendar).unwrap();

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

#[allow(dead_code)]
fn run_experiment_1() {
  database_importer::import_temporary_database().expect("Error");
  let schedule = database_importer::parse_database_data().expect("Failed to import");

  let steps_vec = vec![
    128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576,
    2097152, 4194304, 8388608, 16777216, 33554432, 67108864, 134217728,
  ];

  // TODO: Refactor
  let mp = MultiProgress::new();
  let handles: Vec<std::thread::JoinHandle<Vec<simulation::SimulationOutput>>> = steps_vec
    .into_iter()
    .map(|steps| {
      println!("Starting run for {} steps", steps);

      simulation::generate_schedule(
        vec![SimulationOptions {
          simulation_constraints: schedule.get_simulation_constraints().clone(),
          total_steps: steps,
          initial_state: None,
          temperature_function: simulation::TemperatureFunction::T001,
          progress: simulation::ProgressOption::MultiProgress(mp.clone()),
        }],
        None,
      )
    })
    .collect();

  let results: Vec<simulation::SimulationOutput> = handles
    .into_iter()
    .map(|handle| handle.join().unwrap().into_iter().next().unwrap())
    .collect();

  let file = std::fs::File::create("results.json").unwrap();
  let writer = std::io::BufWriter::new(file);
  serde_json::ser::to_writer(writer, &results).unwrap()
}

#[allow(dead_code)]
fn run_experiment_2() {
  database_importer::import_temporary_database().expect("Error");
  let schedule = database_importer::parse_database_data().expect("Failed to import");

  // let steps = 2097152;
  let steps = 1_000_000;
  let temperature_functions = vec![
    TemperatureFunction::T001,
    TemperatureFunction::T002,
    TemperatureFunction::T003,
  ];

  // TODO: Refactor
  let mp = MultiProgress::new();
  let handles: Vec<std::thread::JoinHandle<Vec<simulation::SimulationOutput>>> =
    temperature_functions
      .into_iter()
      .map(|temperature_function| {
        println!("Starting run for {} steps", steps);

        simulation::generate_schedule(
          vec![SimulationOptions {
            simulation_constraints: schedule.get_simulation_constraints().clone(),
            total_steps: steps,
            initial_state: None,
            temperature_function,
            progress: simulation::ProgressOption::MultiProgress(mp.clone()),
          }],
          None,
        )
      })
      .collect();

  let results: Vec<simulation::SimulationOutput> = handles
    .into_iter()
    .map(|handle| handle.join().unwrap().into_iter().next().unwrap())
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
  // run_experiment_2()
  run_app()
}
