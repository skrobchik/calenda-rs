use egui::{Color32, Layout, Rect, Rounding, Sense, Stroke, Vec2, Widget};
use egui_extras::{Size, StripBuilder};

use crate::school_schedule::SchoolSchedule;
use crate::timeslot;
use crate::week_calendar::Weekday;

pub struct SimpleScheduleWidget<'a> {
  state: &'a SchoolSchedule,
}

impl<'a> SimpleScheduleWidget<'a> {
  pub fn new(state: &'a SchoolSchedule) -> SimpleScheduleWidget<'a> {
    SimpleScheduleWidget {
      state,
    }
  }
  pub fn show(&self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Schedule")
      .open(open)
      .vscroll(false)
      .resizable(true)
      .default_height(500.0)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }
  fn ui(&self, ui: &mut egui::Ui) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
    let total_width = response.rect.width();
    let total_height = response.rect.height();
    let w = total_width / timeslot::DAY_COUNT as f32;
    let h: f32 = total_height / timeslot::TIMESLOT_COUNT as f32;
    for i in timeslot::TIMESLOT_RANGE {
      for j in timeslot::DAY_RANGE {
        let day: Weekday = j.try_into().unwrap();
        let class_data_list = self.state.get_class_data(day, i);
        let num_classes = class_data_list.len();
        let class_width = w / num_classes as f32;
        let mut topleft: egui::Pos2 = response.rect.left_top() + (w * j as f32, h * i as f32).into();
        for class_data in class_data_list {
          let botright: egui::Pos2 = topleft + (class_width, h).into();
          let class_color = class_data.class_metadata.get_color();
          painter.rect(
            Rect::from_two_pos(topleft, botright),
            Rounding::same(0.02 * w.min(h)),
            *class_color,
            Stroke::new(1.0, Color32::from_gray(100)),
          );
          topleft += (class_width, 0.0).into();
        }
      }
    }
  }
}
