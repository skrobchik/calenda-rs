use std::{
  sync::mpsc::{Receiver, Sender},
  thread::JoinHandle,
};

use crate::{
  class_editor::ClassEditor, database_importer, optimization_widget::OptimizationWidget,
  professor_editor::ProfessorEditor, professor_schedule_widget::ProfessorScheduleWidget,
  simple_schedule_widget::SimpleScheduleWidget,
};
use calendars_core::ClassCalendarOptimizer;
use calendars_core::{
  AdvancedSimulationOptions, ClassCalendar, LiveUpdate, ProfessorKey, ProgressOption,
  SchoolSchedule, SimulationOptions, SimulationOutput, TemperatureFunction,
};
use egui::Ui;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tracing::info;

struct CurrentSimulation {
  progress_bar: indicatif::ProgressBar,
  live_update: std::sync::mpsc::Receiver<ClassCalendar>,
  join_handle: JoinHandle<Vec<SimulationOutput>>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MyApp {
  pub school_schedule: SchoolSchedule,
  schedule_widget: SimpleScheduleWidget,
  professor_editor_widget_open: bool,
  class_editor_widget_open: bool,
  class_editor: ClassEditor,
  optimization_widget: OptimizationWidget,
  availability_editor_professor_key: Option<ProfessorKey>,
  availability_editor_widget_open: bool,
  #[serde(skip)]
  current_simulation: Option<CurrentSimulation>,
  pub developer_mode: bool,
}

impl MyApp {
  pub fn new(cc: &eframe::CreationContext<'_>) -> MyApp {
    if let Some(storage) = cc.storage {
      return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
    }
    Default::default()
  }
  fn draw_menu_bar(&mut self, ui: &mut Ui) {
    egui::menu::bar(ui, |ui| {
      ui.menu_button("Archivo", |ui| {
        if ui.button("Restaurar Valores Predeterminados").clicked() {
          *self = MyApp::default();
        }
        if ui.button("Guardar").clicked() {
          if let Some(path) = FileDialog::new()
            .set_title("Guardar Horario Escolar")
            .add_filter("horario", &["horario"])
            .save_file()
          {
            let schedule = self.school_schedule.clone();
            std::fs::write(path, serde_json::to_string(&schedule).unwrap()).unwrap();
          }
        }
        if ui.button("Cargar").clicked() {
          if let Some(path) = FileDialog::new()
            .set_title("Cargar Horario Escolar")
            .add_filter("horario", &["horario"])
            .pick_file()
          {
            let buf = std::fs::read_to_string(path).unwrap();
            let schedule: SchoolSchedule = serde_json::from_str(&buf).unwrap();
            self.school_schedule = schedule;
          }
        }
        if ui.button("Importar SQLs").clicked() {
          let schedule =
            database_importer::import_schedule(Default::default()).expect("Failed to import");
          self.school_schedule = schedule;
        }
      });
      ui.menu_button("Vista", |ui| {
        if ui.button("Editor de Profesores").clicked() {
          self.professor_editor_widget_open = !self.professor_editor_widget_open;
        }
        if ui.button("Editor de Clases").clicked() {
          self.class_editor_widget_open = !self.class_editor_widget_open;
        }
        if ui.button("Calendario").clicked() {
          self
            .schedule_widget
            .open
            .replace(!(self.schedule_widget.open.get()));
        }
      });
    });
  }
}

impl eframe::App for MyApp {
  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    eframe::set_value(storage, eframe::APP_KEY, self);
  }

  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      if self.developer_mode {
        ui.label("DEVELOPER MODE");
        if ui.button("DEV Clear Context Data").clicked() {
          ctx.memory_mut(|mem| {
            *mem = Default::default();
          });
        }
      }

      self.draw_menu_bar(ui);

      if let Some(current_simulation) = self.current_simulation.as_ref() {
        let mut latest_result = None;
        while let Ok(result) = current_simulation.live_update.try_recv() {
          latest_result = Some(result);
        }
        if let Some(latest_result) = latest_result {
          self
            .school_schedule
            .replace_class_calendar(latest_result)
            .unwrap();
        }
      }
      self.schedule_widget.show(ctx, &self.school_schedule);

      self.class_editor.show(ctx, &mut self.school_schedule);

      ProfessorEditor::new(
        &mut self.school_schedule,
        &mut self.availability_editor_professor_key,
        &mut self.availability_editor_widget_open,
      )
      .show(ctx, &mut self.professor_editor_widget_open);

      if let Some(professor_id) = self.availability_editor_professor_key {
        if let Some(professor) = self.school_schedule.get_professor_mut(professor_id) {
          ProfessorScheduleWidget::new(professor)
            .show(ctx, &mut self.availability_editor_widget_open);
        }
      }

      if let Some(stop_condition) = self.optimization_widget.show(
        ctx,
        self.current_simulation.as_ref().map(|x| &x.progress_bar),
      ) {
        self.current_simulation = {
          let (live_update_simulation_tx, live_update_proxy_rx): (
            Sender<ClassCalendar>,
            Receiver<ClassCalendar>,
          ) = std::sync::mpsc::channel();
          let (live_update_proxy_tx, live_update_app_rx): (
            Sender<ClassCalendar>,
            Receiver<ClassCalendar>,
          ) = std::sync::mpsc::channel();
          let advanced_options = AdvancedSimulationOptions {
            live_update: Some(LiveUpdate {
              channel: live_update_simulation_tx,
              live_update_interval: 5_000,
            }),
            ..Default::default()
          };
          let progress_bar = indicatif::ProgressBar::hidden();
          let pb = progress_bar.clone();
          let local_simulation_constraints =
            self.school_schedule.get_simulation_constraints().clone();
          let local_ctx = ctx.clone();
          let initial_state = self.school_schedule.get_class_calendar().clone();
          let join_handle = std::thread::spawn(move || {
            let pb2 = pb.clone();
            let pb_ctx = local_ctx.clone();
            let pb_thread = std::thread::spawn(move || {
              let mut p = pb2.position();
              pb_ctx.request_repaint();
              loop {
                if pb2.position() != p {
                  p = pb2.position();
                  pb_ctx.request_repaint();
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
              }
            });
            let live_update_ctx = local_ctx.clone();
            let live_update_proxy_thread = std::thread::spawn(move || {
              while let Ok(val) = live_update_proxy_rx.recv() {
                match live_update_proxy_tx.send(val) {
                  Ok(_) => live_update_ctx.request_repaint(), // live update sent succesfully, issue repaint
                  Err(_) => break, // app thread channel no longer active, exit thread
                }
              }
              // simulation thread channel no longer active, exit thread
            });
            let mut optimizer = calendars_core::SimulatedAnnealingOptimizer::default();
            let simulation_thread: JoinHandle<Vec<SimulationOutput>> =
              std::thread::spawn(move || {
                let options = SimulationOptions {
                  stop_condition,
                  initial_state,
                  temperature_function: TemperatureFunction::Linear,
                  progress: ProgressOption::ProgressBar(pb),
                  advanced_options,
                };
                let class_calendar = optimizer.generate_class_calendar(
                  local_simulation_constraints,
                  options.clone(),
                  None,
                );
                let simulation_output: SimulationOutput = SimulationOutput {
                  simulation_options: options,
                  final_calendar: Default::default(),
                  final_cost: Default::default(),
                  start_time: std::time::SystemTime::UNIX_EPOCH,
                  end_time: std::time::SystemTime::UNIX_EPOCH,
                  duration: Default::default(),
                  stats: Default::default(),
                  total_steps: Default::default(),
                  classroom_assignments: Default::default(),
                };
                vec![simulation_output]
              });
            let r = simulation_thread.join().unwrap();
            drop(pb_thread);
            drop(live_update_proxy_thread);
            local_ctx.request_repaint();
            r
          });
          Some(CurrentSimulation {
            progress_bar,
            join_handle,
            live_update: live_update_app_rx,
          })
        };
      }

      if self.current_simulation.is_some() {
        let is_finished = self
          .current_simulation
          .as_ref()
          .unwrap()
          .join_handle
          .is_finished();
        if is_finished {
          let simulation_output = self.current_simulation.take();
          let simulation_output = simulation_output.unwrap().join_handle.join().unwrap();
          save_latest_simulation_output(&simulation_output).unwrap();
          let new_class_calendar = simulation_output.into_iter().nth(0).unwrap().final_calendar;
          self
            .school_schedule
            .replace_class_calendar(new_class_calendar)
            .unwrap();
          info!("Applied new schedule");
        }
      }
    });
  }
}

fn save_latest_simulation_output(simulation_output: &Vec<SimulationOutput>) -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;
  let cwd = std::path::Path::new(&cwd);
  let t = chrono::Local::now();
  let file_path = cwd.join(format!(
    "simulation_results_{}.pickle",
    t.format("%FT%H%M%S")
  ));
  let file = std::fs::File::create(file_path)?;
  let mut writer = std::io::BufWriter::new(file);
  let ser_options = serde_pickle::SerOptions::default();
  serde_pickle::to_writer(&mut writer, simulation_output, ser_options)?;
  Ok(())
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      class_editor_widget_open: true,
      professor_editor_widget_open: true,
      school_schedule: Default::default(),
      availability_editor_professor_key: None,
      availability_editor_widget_open: true,
      current_simulation: None,
      class_editor: Default::default(),
      optimization_widget: Default::default(),
      developer_mode: false,
      schedule_widget: Default::default(),
    }
  }
}
