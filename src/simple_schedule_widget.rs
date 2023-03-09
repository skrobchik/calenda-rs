use egui::{Color32, Layout, Rect, Rounding, Sense, Stroke, Vec2, Widget};
use egui_extras::{Size, StripBuilder};

use crate::school_schedule::SchoolSchedule;
use crate::timeslot;

struct Style {
  timeslot_width: f32,
  timeslot_height: f32,
}

impl Default for Style {
  fn default() -> Self {
    Self {
      timeslot_width: 30.0,
      timeslot_height: 10.0,
    }
  }
}

pub struct SimpleScheduleWidget<'a> {
  state: &'a SchoolSchedule,
  style: Style,
}

impl<'a> SimpleScheduleWidget<'a> {
  pub fn new(state: &'a SchoolSchedule) -> SimpleScheduleWidget<'a> {
    SimpleScheduleWidget {
      state,
      style: Default::default(),
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
    let W = response.rect.width();
    let H = response.rect.height();
    let w = W / timeslot::DAY_COUNT as f32;
    let h: f32 = H / timeslot::TIMESLOT_COUNT as f32;
    for i in 0..timeslot::TIMESLOT_COUNT {
      for j in 0..timeslot::DAY_COUNT {
        let topleft: egui::Pos2 = response.rect.left_top() + (w * j as f32, h * i as f32).into();
        let botright: egui::Pos2 = topleft + (w, h).into();
        painter.rect(
          Rect::from_two_pos(topleft, botright),
          Rounding::same(0.02 * w.min(h)),
          Color32::from_gray(200),
          Stroke::new(1.0, Color32::from_gray(100)),
        )
      }
    }
  }
}
