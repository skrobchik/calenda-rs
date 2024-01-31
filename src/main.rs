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
pub mod timeslot;
pub mod week_calendar;

use std::time::Duration;

use crate::app::MyApp;

use indicatif::{MultiProgress, ProgressStyle};
use itertools::Itertools;
use simulation_options::{SimulationOptions, TemperatureFunction};
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
  // let simulation_output = load_results("results3.json").into_iter().nth(20).unwrap();
  // println!("Num Steps: {}", simulation_output.best_schedule_run_report.num_steps);
  // println!("Cost: {}", simulation_output.best_schedule_cost);
  // let class_calendar = simulation_output.best_schedule;
  // schedule.replace_class_calendar(class_calendar).unwrap();

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
      temperature_function: simulation_options::TemperatureFunction::T001,
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

#[allow(dead_code)]
fn run_experiment_2() {
  let schedule = database_importer::import_schedule(Default::default()).expect("Failed to import");

  let total_steps = 1_000_000;
  let temperature_functions = vec![
    TemperatureFunction::T001,
    TemperatureFunction::T002,
    TemperatureFunction::T003,
  ];

  // TODO: Refactor
  let mp = MultiProgress::new();

  let simulation_options: Vec<SimulationOptions> = temperature_functions
    .into_iter()
    .map(|temperature_function| SimulationOptions {
      simulation_constraints: schedule.get_simulation_constraints().clone(),
      stop_condition: simulation_options::StopCondition::Steps(total_steps),
      initial_state: None,
      progress: simulation_options::ProgressOption::MultiProgress(mp.clone()),
      temperature_function,
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

#[allow(dead_code)]
fn run_experiment_3() {
  let schedule = database_importer::import_schedule(Default::default()).expect("Failed to import");

  let total_steps = 2_000_000;

  let progress_bar_update_interval_list = vec![1, 2, 10, 100, 1000, 20_000];

  let mp = MultiProgress::new();

  let simulation_options: Vec<SimulationOptions> = progress_bar_update_interval_list
    .into_iter()
    .map(|progress_bar_update_interval| {
      let progress_bar_style = ProgressStyle::with_template(
        "{prefix} progress_bar_update_interval {spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {human_pos}/{human_len} ({percent} %) ({eta}) ({per_sec})",
      )
      .unwrap()
      .progress_chars("#>-");

      let pb = indicatif::ProgressBar::new(total_steps as u64).with_style(progress_bar_style);
      let pb = mp.add(pb);
      pb.set_prefix(progress_bar_update_interval.to_string());

      SimulationOptions {
        simulation_constraints: schedule.get_simulation_constraints().clone(),
        stop_condition: simulation_options::StopCondition::Steps(total_steps),
        initial_state: None,
        progress: simulation_options::ProgressOption::ProgressBar(pb.clone()),
        temperature_function: simulation_options::TemperatureFunction::T001,
        advanced_options: simulation_options::AdvancedSimulationOptions { progress_bar_update_interval },
      }
    })
    .collect();

  let results = simulation::generate_schedule(simulation_options, None)
    .join()
    .unwrap();

  let file = std::fs::File::create("results.json").unwrap();
  let writer = std::io::BufWriter::new(file);
  serde_json::ser::to_writer(writer, &results).unwrap()
}

#[allow(dead_code)]
fn run_experiment_4() {
  let schedule = database_importer::import_schedule(Default::default()).expect("Failed to import");

  let simulation_options: Vec<SimulationOptions> = vec![SimulationOptions {
    simulation_constraints: schedule.get_simulation_constraints().clone(),
    stop_condition: simulation_options::StopCondition::Time(Duration::from_secs(60 * 60)),
    initial_state: None,
    progress: simulation_options::ProgressOption::MultiProgress(MultiProgress::new()),
    temperature_function: simulation_options::TemperatureFunction::T001,
    advanced_options: Default::default(),
  }];

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

  // run_experiment_1()
  // run_experiment_2()
  // run_experiment_3()
  // run_experiment_4()
  run_app(developer_mode)
}
