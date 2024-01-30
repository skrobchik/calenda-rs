use std::thread::JoinHandle;

use crate::{
  class_editor::ClassEditor,
  class_filter::ClassFilter,
  database_importer,
  optimization_widget::OptimizationWidget,
  professor_editor::ProfessorEditor,
  professor_schedule_widget::ProfessorScheduleWidget,
  school_schedule::{SchoolSchedule, Semester},
  simple_schedule_widget::SimpleScheduleWidget,
  simulation::{self, SimulationOutput},
  simulation_options::{self, SimulationOptions},
};
use eframe::egui;
use egui::Ui;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tracing::info;

struct CurrentSimulation {
  progress_bar: indicatif::ProgressBar,
  join_handle: JoinHandle<Vec<SimulationOutput>>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct MyApp {
  pub(crate) school_schedule: SchoolSchedule,
  schedule_widget_open: bool,
  professor_editor_widget_open: bool,
  class_editor_widget_open: bool,
  class_editor: ClassEditor,
  optimization_widget: OptimizationWidget,
  availability_editor_professor_id: Option<usize>,
  availability_editor_widget_open: bool,
  schedule_widget_filter: ClassFilter,
  #[serde(skip)]
  current_simulation: Option<CurrentSimulation>,
}

impl MyApp {
  pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> MyApp {
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
        if ui.button("Exportar").clicked() {
          if let Some(path) = FileDialog::new()
            .set_title("Exportar Calendario")
            .add_filter("ics", &["ics"])
            .save_file()
          {
            std::fs::write(path, self.school_schedule.export_ics().to_string()).unwrap();
          }
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
      });
      ui.menu_button("Filtro", |ui| {
        for (semester, semester_name) in [
          Semester::S1,
          Semester::S2,
          Semester::S3,
          Semester::S4,
          Semester::S5,
          Semester::S6,
          Semester::S7,
          Semester::S8,
        ]
        .iter()
        .zip(
          [
            "Semestre 1",
            "Semestre 2",
            "Semestre 3",
            "Semestre 4",
            "Semestre 5",
            "Semestre 6",
            "Semestre 7",
            "Semestre 8",
          ]
          .iter(),
        ) {
          if ui.button(semester_name.to_string()).clicked() {
            self.schedule_widget_filter = ClassFilter::Semester(*semester);
          }
        }
        if ui.button("Todos").clicked() {
          self.schedule_widget_filter = ClassFilter::None;
        }
      });
      ui.menu_button("Load results3.json", |ui| {
        (0..21).for_each(|i| {
          if ui.button(i.to_string()).clicked() {
            let simulation_output = crate::load_results("results3.json")
              .into_iter()
              .nth(i)
              .unwrap();
            // println!(
            //   "Num Steps: {}",
            //   simulation_output.simulation_options.total_steps
            // );
            println!("Cost: {}", simulation_output.final_cost);
            let class_calendar = simulation_output.final_calendar;
            self
              .school_schedule
              .replace_class_calendar(class_calendar)
              .unwrap();
          }
        })
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
      self.draw_menu_bar(ui);

      SimpleScheduleWidget::new(&self.school_schedule, self.schedule_widget_filter.clone())
        .show(ctx, &mut self.schedule_widget_open);

      self.class_editor.show(ctx, &mut self.school_schedule);

      ProfessorEditor::new(
        &mut self.school_schedule,
        &mut self.availability_editor_professor_id,
        &mut self.availability_editor_widget_open,
      )
      .show(ctx, &mut self.professor_editor_widget_open);

      if let Some(professor_id) = self.availability_editor_professor_id {
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
          let progress_bar = indicatif::ProgressBar::hidden();
          let local_progress_bar = progress_bar.clone();
          let local_simulation_constraints =
            self.school_schedule.get_simulation_constraints().clone();
          let local_ctx = ctx.clone();
          let join_handle = std::thread::spawn(move || {
            let local_progress_bar2 = local_progress_bar.clone();
            let local_ctx2 = local_ctx.clone();
            let h2 = std::thread::spawn(move || {
              let mut p = local_progress_bar2.position();
              local_ctx2.request_repaint();
              loop {
                if local_progress_bar2.position() != p {
                  p = local_progress_bar2.position();
                  local_ctx2.request_repaint();
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
              }
            });
            let h1: JoinHandle<Vec<SimulationOutput>> = simulation::generate_schedule(
              vec![SimulationOptions {
                simulation_constraints: local_simulation_constraints,
                stop_condition,
                initial_state: None,
                temperature_function: simulation_options::TemperatureFunction::T001,
                progress: simulation_options::ProgressOption::ProgressBar(local_progress_bar),
                advanced_options: Default::default(),
              }],
              None,
            );
            let r = h1.join().unwrap();
            drop(h2);
            local_ctx.request_repaint();
            r
          });
          Some(CurrentSimulation {
            progress_bar,
            join_handle,
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

impl Default for MyApp {
  fn default() -> Self {
    Self {
      schedule_widget_open: true,
      class_editor_widget_open: true,
      professor_editor_widget_open: true,
      school_schedule: Default::default(),
      availability_editor_professor_id: None,
      availability_editor_widget_open: true,
      current_simulation: None,
      schedule_widget_filter: ClassFilter::None,
      class_editor: Default::default(),
      optimization_widget: Default::default(),
    }
  }
}
