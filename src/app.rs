use crate::{
  metadata_register::SemesterNumber,
  school_schedule::SchoolSchedule,
  simple_schedule_widget::SimpleScheduleWidget,
};
use eframe::egui;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Serialize, Deserialize, Debug)]
enum CalendarView {
  All,
  Semester(SemesterNumber),
  EvenSemesters,
  OddSemesters,
  Professor(usize),
  Class(usize),
}

impl Default for CalendarView {
  fn default() -> Self {
    CalendarView::All
  }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MyApp {
  school_schedule: SchoolSchedule,
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
      });
      ui.menu_button("Vista", |ui| {
        if ui.button("Editor de Profesores").clicked() {
        }
        if ui.button("Editor de Parametros de Simulacion").clicked() {
        }
        if ui.button("Editor de Clases").clicked() {
        }
      });
    });
  }
}

impl eframe::App for MyApp {
  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    eframe::set_value(storage, eframe::APP_KEY, self);
  }

  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      self.draw_menu_bar(ui);

      ui.add(SimpleScheduleWidget::new(&self.school_schedule));

    });

    // Resize the native window to be just the size we need it to be:
    frame.set_window_size(ctx.used_size());
  }
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      school_schedule: Default::default(),
    }
  }
}
