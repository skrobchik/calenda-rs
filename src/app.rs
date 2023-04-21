use crate::{
  class_editor::ClassEditor,
  professor_schedule_widget::ProfessorScheduleWidget,
  school_schedule::{Professor, SchoolSchedule},
  simple_schedule_widget::SimpleScheduleWidget,
};
use eframe::egui;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MyApp {
  school_schedule: SchoolSchedule,
  schedule_widget_open: bool,
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
        if ui.button("Editor de Profesores").clicked() {}
        if ui.button("Editor de Parametros de Simulacion").clicked() {}
        if ui.button("Editor de Clases").clicked() {}
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

      SimpleScheduleWidget::new(&self.school_schedule).show(ctx, &mut self.schedule_widget_open);

      ClassEditor::new(&mut self.school_schedule).show(ctx, &mut self.schedule_widget_open);

      if let Some(professor) = &mut self.school_schedule.simulation_information.professors[0] {
        ProfessorScheduleWidget::new(professor).show(ctx, &mut self.schedule_widget_open);
      } else {
        let professor: Professor = Default::default();
        self.school_schedule.simulation_information.professors[0] = Some(professor);
      }
    });

    // Resize the native window to be just the size we need it to be:
    frame.set_window_size(ctx.used_size());
  }
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      school_schedule: Default::default(),
      schedule_widget_open: true,
    }
  }
}
