use egui::{Color32, Rect, Rounding, Sense, Stroke, Vec2, Widget, Layout};
use egui_extras::{StripBuilder, Size};

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
  pub fn ui(&self, ui: &mut egui::Ui) {
    StripBuilder::new(ui).size(Size::exact(50.0)).vertical(|mut strip| {});
  }
}