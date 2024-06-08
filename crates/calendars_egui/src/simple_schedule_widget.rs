use std::cell::Cell;

use calendars_core::{
  ClassFilter, Classroom, Day, ProfessorKey, SchoolSchedule, Semester, Timeslot,
};
use egui::{Align2, Color32, FontId, Rect, Rounding, Sense, Stroke};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SimpleScheduleWidget {
  pub class_filter: ClassFilter,
  pub open: Cell<bool>,
}

impl SimpleScheduleWidget {
  pub fn show(&mut self, ctx: &egui::Context, state: &SchoolSchedule) {
    let mut local_open = self.open.clone();
    egui::Window::new("Horario")
      .open(local_open.get_mut())
      .vscroll(false)
      .resizable(true)
      .default_height(500.0)
      .show(ctx, |ui| {
        self.ui(ui, state);
      });
  }
  fn ui_calendar(&self, ui: &mut egui::Ui, state: &SchoolSchedule) {
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), Sense::hover());
    let total_width = response.rect.width();
    let total_height = response.rect.height();
    let w = total_width / Day::all().len() as f32;
    let h: f32 = total_height / Timeslot::all().len() as f32;
    let mut first: bool = true;
    for day in Day::all() {
      for timeslot in Timeslot::all() {
        let classes_to_draw = state
          .get_class_calendar()
          .iter_class_keys()
          .filter(|k| {
            let b = self.class_filter.filter(
              *k,
              state.get_simulation_constraints(),
              state.get_class_calendar(),
              day,
              timeslot,
              first,
            );
            first = false;
            b
          })
          .collect_vec();

        let total_count: u32 = classes_to_draw
          .iter()
          .map(|k| state.get_class_calendar().get_count(day, timeslot, *k) as u32)
          .sum();

        let class_width = w / total_count as f32;

        let mut topleft: egui::Pos2 = response.rect.left_top()
          + (
            w * usize::from(day) as f32,
            h * usize::from(timeslot) as f32,
          )
            .into();

        painter.rect_stroke(
          Rect::from_two_pos(topleft, topleft + (w, h).into()),
          Rounding::ZERO,
          Stroke::new(1.0, Color32::from_gray(100)),
        );

        for (class_key, class_count) in classes_to_draw
          .iter()
          .map(|&k| (k, state.get_class_calendar().get_count(day, timeslot, k)))
        {
          if let Some(class_metadata) = state.get_class_metadata(class_key) {
            for _ in 0..class_count {
              let botright: egui::Pos2 = topleft + (class_width, h).into();
              let rgba = class_metadata.rgba;
              let class_color =
                Color32::from_rgba_premultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
              painter.rect(
                Rect::from_two_pos(topleft, botright),
                Rounding::same(0.02 * w.min(h)),
                class_color,
                Stroke::new(1.0, Color32::from_gray(100)),
              );
              let class_code = &class_metadata.class_code;
              painter.text(
                topleft,
                Align2::LEFT_TOP,
                class_code,
                FontId::default(),
                Color32::BLACK,
              );
              topleft += (class_width, 0.0).into();
            }
          }
        }
      }
    }
  }
  fn ui_control_export(&mut self, ui: &mut egui::Ui, state: &SchoolSchedule) {
    if ui.button("Exportar").clicked() {
      if let Some(path) = rfd::FileDialog::new()
        .set_title("Exportar Calendario")
        .add_filter("ics", &["ics"])
        .save_file()
      {
        std::fs::write(path, state.export_ics(&self.class_filter).to_string()).unwrap();
      }
    }
  }
  fn ui_control_filters(&mut self, ui: &mut egui::Ui, state: &SchoolSchedule) {
    if ui
      .radio(matches!(self.class_filter, ClassFilter::None), "Todo")
      .clicked()
    {
      self.class_filter = ClassFilter::None;
    }

    ui.horizontal(|ui| {
      if ui
        .radio(
          matches!(self.class_filter, ClassFilter::Semester(_)),
          "Semestre",
        )
        .clicked()
        && !matches!(self.class_filter, ClassFilter::Semester(_))
      {
        self.class_filter = ClassFilter::Semester(Semester::S1);
      }
      if let ClassFilter::Semester(semester) = &mut self.class_filter {
        egui::ComboBox::new("schedule_widget_combo_box_1", "")
          .selected_text(semester.to_string())
          .show_ui(ui, |ui| {
            ui.selectable_value(semester, Semester::S1, Semester::S1.to_string());
            ui.selectable_value(semester, Semester::S2, Semester::S2.to_string());
            ui.selectable_value(semester, Semester::S3, Semester::S3.to_string());
            ui.selectable_value(semester, Semester::S4, Semester::S4.to_string());
            ui.selectable_value(semester, Semester::S5, Semester::S5.to_string());
            ui.selectable_value(semester, Semester::S6, Semester::S6.to_string());
            ui.selectable_value(semester, Semester::S7, Semester::S7.to_string());
            ui.selectable_value(semester, Semester::S8, Semester::S8.to_string());
          });
      }
    });

    ui.horizontal(|ui| {
      let professor_key = state.get_simulation_constraints().professors.keys().next();
      ui.add_enabled_ui(professor_key.is_some(), |ui| {
        if ui
          .radio(
            matches!(self.class_filter, ClassFilter::Professor(_)),
            "Profesor",
          )
          .clicked()
          && !matches!(self.class_filter, ClassFilter::Professor(_))
        {
          self.class_filter =
            ClassFilter::Professor(professor_key.expect("Radio shouldn't be enabled"));
        }
      });
      if let ClassFilter::Professor(professor_key) = &mut self.class_filter {
        egui::ComboBox::new("schedule_widget_combo_box_2", "")
          .selected_text(
            state
              .get_professor_metadata(*professor_key)
              .map(|professor| professor.name.clone())
              .unwrap_or("Profesor Inexistente".to_string()),
          )
          .show_ui(ui, |ui| {
            let professor_keys: Vec<ProfessorKey> = state
              .get_simulation_constraints()
              .professors
              .keys()
              .collect();
            for i in professor_keys {
              ui.selectable_value(
                professor_key,
                i,
                state.get_professor_metadata(i).unwrap().name.clone(),
              );
            }
          });
      }
    });

    ui.horizontal(|ui| {
      if ui
        .radio(
          matches!(self.class_filter, ClassFilter::Classroom(_)),
          "Aula",
        )
        .clicked()
        && !matches!(self.class_filter, ClassFilter::Classroom(_))
      {
        self.class_filter = ClassFilter::Classroom(Classroom::Aula5_6);
      }
      if let ClassFilter::Classroom(classroom) = &mut self.class_filter {
        egui::ComboBox::new("schedule_widget_combo_box_2", "")
          .selected_text(classroom.to_string())
          .show_ui(ui, |ui| {
            ui.selectable_value(classroom, Classroom::Aula1, Classroom::Aula1.to_string());
            ui.selectable_value(
              classroom,
              Classroom::Aula2_3,
              Classroom::Aula2_3.to_string(),
            );
            ui.selectable_value(classroom, Classroom::Aula4, Classroom::Aula4.to_string());
            ui.selectable_value(
              classroom,
              Classroom::Aula5_6,
              Classroom::Aula5_6.to_string(),
            );
            ui.selectable_value(
              classroom,
              Classroom::SalaSeminarios,
              Classroom::SalaSeminarios.to_string(),
            );
            ui.selectable_value(
              classroom,
              Classroom::SalaComputo,
              Classroom::SalaComputo.to_string(),
            );
            ui.selectable_value(
              classroom,
              Classroom::LabFisica,
              Classroom::LabFisica.to_string(),
            );
            ui.selectable_value(
              classroom,
              Classroom::LabQuimica,
              Classroom::LabQuimica.to_string(),
            );
            ui.selectable_value(
              classroom,
              Classroom::NotAssigned,
              "No Asignados (choque de clases)",
            )
          });
      }
    });
  }
  fn ui_control(&mut self, ui: &mut egui::Ui, state: &SchoolSchedule) {
    ui.horizontal(|ui| {
      ui.vertical(|ui| {
        self.ui_control_filters(ui, state);
      });
      ui.vertical(|ui| {
        self.ui_control_export(ui, state);
      })
    });
  }

  fn ui(&mut self, ui: &mut egui::Ui, state: &SchoolSchedule) {
    self.ui_control(ui, state);
    ui.separator();
    self.ui_calendar(ui, state);
  }
}
