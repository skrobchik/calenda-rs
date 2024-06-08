pub mod app;
pub mod class_editor;
pub mod color_list;
pub mod database_importer;
pub mod optimization_widget;
pub mod professor_editor;
pub mod professor_schedule_widget;
pub mod simple_schedule_widget;

use crate::app::MyApp;

use calendars_core::SimulationOutput;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

pub fn load_results<P: AsRef<std::path::Path>>(path: P) -> Vec<SimulationOutput> {
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

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let developer_mode: bool = args.iter().any(|x| x == "-d");

  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::DEBUG)
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

  run_app(developer_mode)
}
