pub mod app;
pub mod class_editor;
pub mod class_filter;
pub mod color_list;
pub mod database_importer;
pub mod heuristics;
pub mod optimization_widget;
pub mod professor_editor;
pub mod professor_schedule_widget;
pub mod school_schedule;
pub mod simple_schedule_widget;
pub mod simulation;
pub mod simulation_options;
pub mod stats_tracker;
pub mod week_calendar;

use crate::app::MyApp;

use indicatif::MultiProgress;
use simulation_options::SimulationOptions;
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
fn run_app(developer_mode: bool) {
  let options = eframe::NativeOptions::default();
  eframe::run_native(
    "my_app",
    options,
    Box::new(move |cc| {
      let mut x = Box::new(MyApp::new(cc));
      x.developer_mode = developer_mode;
      x
    }),
  )
  .expect("Something went wrong!");
}

#[allow(dead_code)]
fn run_experiment_1() {
  let schedule = database_importer::import_schedule(Default::default()).expect("Failed to import");

  let max_step_limit = 60 * 28000;
  let steps_vec = (0..u32::MAX).map_while(|p| {
    let steps = 2_i32.pow(p);
    if steps <= max_step_limit {
      Some(steps)
    } else {
      None
    }
  });

  let mp = MultiProgress::new();

  let simulation_options: Vec<SimulationOptions> = steps_vec
    .into_iter()
    .map(|total_steps| SimulationOptions {
      simulation_constraints: schedule.get_simulation_constraints().clone(),
      stop_condition: simulation_options::StopCondition::Steps(total_steps as usize),
      initial_state: None,
      progress: simulation_options::ProgressOption::MultiProgress(mp.clone()),
      temperature_function: simulation_options::TemperatureFunction::Linear,
      advanced_options: Default::default(),
    })
    .collect();

  let results = simulation::generate_schedule(simulation_options, None)
    .join()
    .unwrap();

  let file = std::fs::File::create("results.json").unwrap();
  let writer = std::io::BufWriter::new(file);
  serde_json::ser::to_writer(writer, &results).unwrap()
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let developer_mode: bool = args.iter().any(|x| x == "-d");

  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

  run_app(developer_mode)
  // run_experiment_1()
}
