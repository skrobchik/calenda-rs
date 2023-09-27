use egui::{Color32, Rect, Rounding, Sense, Stroke};

use crate::school_schedule::SchoolSchedule;
use crate::timeslot;

pub(crate) struct SimpleScheduleWidget<'a> {
  state: &'a SchoolSchedule,
}

impl<'a> SimpleScheduleWidget<'a> {
  pub(crate) fn new(state: &'a SchoolSchedule) -> SimpleScheduleWidget<'a> {
    SimpleScheduleWidget { state }
  }
  pub(crate) fn show(&self, ctx: &egui::Context, open: &mut bool) {
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
    for day_idx in timeslot::DAY_RANGE {
      for timeslot_idx in timeslot::TIMESLOT_RANGE {
        let timeslot = self
          .state
          .get_class_calendar()
          .get_timeslot(day_idx, timeslot_idx);

        let num_sessions: u32 = timeslot.iter().map(|x| *x as u32).sum();

        let class_width = w / num_sessions as f32;

        let mut topleft: egui::Pos2 =
          response.rect.left_top() + (w * day_idx as f32, h * timeslot_idx as f32).into();
          
        painter.rect_stroke(Rect::from_two_pos(topleft, topleft+(w,h).into()), Rounding::none(), Stroke::new(1.0, Color32::from_gray(100)));
        
        for class_id in 0..timeslot.len() {
          let class_metadata = self.state.get_class_metadata(class_id).unwrap();
          for _ in 0..timeslot[class_id] {
            let botright: egui::Pos2 = topleft + (class_width, h).into();
            let class_color = class_metadata.color;
            painter.rect(
              Rect::from_two_pos(topleft, botright),
              Rounding::same(0.02 * w.min(h)),
              class_color,
              Stroke::new(1.0, Color32::from_gray(100)),
            );
            topleft += (class_width, 0.0).into();
          }
        }
      }
    }
  }
}
