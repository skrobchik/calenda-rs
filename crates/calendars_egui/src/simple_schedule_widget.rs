use std::cell::Cell;

use calendars_core::{
  ClassFilter, Classroom, Day, SchoolSchedule, Semester, Timeslot, DAY_COUNT, TIMESLOT_COUNT,
};
use egui::{Align2, Color32, FontId, Rect, Rounding, Sense, Stroke};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SimpleScheduleWidget {
  pub class_filter: ClassFilter,
  pub open: Cell<bool>,
}

impl SimpleScheduleWidget {
  pub fn show(&mut self, ctx: &egui::Context, state: &SchoolSchedule) {
    let mut local_open = self.open.clone();
    egui::Window::new("Schedule")
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
    let w = total_width / DAY_COUNT as f32;
    let h: f32 = total_height / TIMESLOT_COUNT as f32;
    let mut first = true;
    for day_idx in Day::all() {
      for timeslot_idx in Timeslot::all() {
        let timeslot: Vec<u8> = state
          .get_class_calendar()
          .get_timeslot(day_idx, timeslot_idx)
          .iter()
          .enumerate()
          .map(|(class_id, count)| {
            let c = if self.class_filter.filter(
              class_id.try_into().unwrap(),
              state.get_simulation_constraints(),
              state.get_class_calendar(),
              day_idx,
              timeslot_idx,
              first,
            ) {
              *count
            } else {
              0
            };
            first = false;
            c
          })
          .collect();

        let num_sessions: u32 = timeslot.iter().map(|x| *x as u32).sum();

        let class_width = w / num_sessions as f32;

        let mut topleft: egui::Pos2 = response.rect.left_top()
          + (
            w * usize::from(day_idx) as f32,
            h * usize::from(timeslot_idx) as f32,
          )
            .into();

        painter.rect_stroke(
          Rect::from_two_pos(topleft, topleft + (w, h).into()),
          Rounding::ZERO,
          Stroke::new(1.0, Color32::from_gray(100)),
        );

        for (class_id, class_count) in timeslot.iter().enumerate() {
          if let Some(class_metadata) = state.get_class_metadata(class_id.try_into().unwrap()) {
            for _ in 0..*class_count {
              let botright: egui::Pos2 = topleft + (class_width, h).into();
              let class_color = class_metadata.color;
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
      if ui
        .radio(
          matches!(self.class_filter, ClassFilter::ProfessorId(_)),
          "Profesor",
        )
        .clicked()
        && !matches!(self.class_filter, ClassFilter::ProfessorId(_))
      {
        self.class_filter = ClassFilter::ProfessorId(0);
      }
      if let ClassFilter::ProfessorId(professor_id) = &mut self.class_filter {
        egui::ComboBox::new("schedule_widget_combo_box_2", "")
          .selected_text(
            state
              .get_professor_metadata(*professor_id)
              .map(|professor| professor.name.clone())
              .unwrap_or("Profesor Inexistente".to_string()),
          )
          .show_ui(ui, |ui| {
            for i in 0..state.get_num_professors() {
              ui.selectable_value(
                professor_id,
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
