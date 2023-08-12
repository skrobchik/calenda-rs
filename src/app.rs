use std::{mem, thread::JoinHandle};

use crate::{
  class_editor::ClassEditor,
  professor_editor::ProfessorEditor,
  professor_schedule_widget::ProfessorScheduleWidget,
  school_schedule::{SchoolSchedule, TimeslotClassHours},
  simple_schedule_widget::SimpleScheduleWidget,
  simulation,
  week_calendar::WeekCalendar,
};
use eframe::egui;
use egui::Ui;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct MyApp {
  pub(crate) school_schedule: SchoolSchedule,
  schedule_widget_open: bool,
  professor_editor_widget_open: bool,
  class_editor_widget_open: bool,
  availability_editor_professor_id: Option<usize>,
  availability_editor_widget_open: bool,
  #[serde(skip)]
  new_schedule_join_handle: Option<JoinHandle<WeekCalendar<TimeslotClassHours>>>,
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
      });
      ui.menu_button("Vista", |ui| {
        if ui.button("Editor de Profesores").clicked() {
          self.professor_editor_widget_open = !self.professor_editor_widget_open;
        }
        if ui.button("Editor de Clases").clicked() {
          self.class_editor_widget_open = !self.class_editor_widget_open;
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

      SimpleScheduleWidget::new(&self.school_schedule).show(ctx, &mut self.schedule_widget_open);

      ClassEditor::new(&mut self.school_schedule).show(ctx, &mut self.class_editor_widget_open);

      ProfessorEditor::new(
        &mut self.school_schedule,
        &mut self.availability_editor_professor_id,
        &mut self.availability_editor_widget_open,
      )
      .show(ctx, &mut self.professor_editor_widget_open);

      if let Some(professor_id) = self.availability_editor_professor_id {
        if let Some(professor) = self
          .school_schedule
          .simulation_information
          .professors
          .get_mut(professor_id)
        {
          ProfessorScheduleWidget::new(professor)
            .show(ctx, &mut self.availability_editor_widget_open);
        }
      }

      if self.new_schedule_join_handle.is_some() {
        ui.label("Optimizing...");
        let is_finished = self
          .new_schedule_join_handle
          .as_ref()
          .unwrap()
          .is_finished();
        if is_finished {
          let new_schedule = mem::take(&mut self.new_schedule_join_handle)
            .unwrap()
            .join()
            .unwrap();
          info!("Applied new schedule");
          self.school_schedule.schedule = new_schedule;
          self.school_schedule.fill_classes();
        }
      } else if ui.button("Optimize").clicked() {
        self.new_schedule_join_handle = Some(simulation::generate_schedule(
          self.school_schedule.simulation_information.clone(),
        ));
      }
    });

    // Resize the native window to be just the size we need it to be:
    frame.set_window_size(ctx.used_size());
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
      new_schedule_join_handle: None,
    }
  }
}
