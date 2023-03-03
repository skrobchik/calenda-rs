use egui::{Color32, Rect, Rounding, Sense, Stroke, Vec2, Widget};

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
  pub fn new(calendar_state: &'a SchoolSchedule) -> SimpleScheduleWidget<'a> {
    SimpleScheduleWidget {
      state: calendar_state,
      style: Default::default(),
    }
  }
}

impl Widget for SimpleScheduleWidget<'_> {
  fn ui(self, ui: &mut egui::Ui) -> egui::Response {
    let w = self.style.timeslot_width;
    let h = self.style.timeslot_height;
    let desired_size = Vec2::new(
      timeslot::DAY_COUNT as f32 * w,
      timeslot::TIMESLOT_COUNT as f32 * h,
    );
    let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
    let rect = response.rect;

    for day in timeslot::DAY_RANGE {
      for timeslot in timeslot::TIMESLOT_RANGE {
        let classes = self.state.get_classes(day.try_into().unwrap(), timeslot);

        let x0 = rect.left_top().x;
        let y0 = rect.left_top().y;
        let x1 = x0 + w * (day as f32);
        let y1 = y0 + h * (timeslot as f32);
        let x2 = x1 + w;
        let y2 = y1 + h;

        let num_classes = classes.len();
        let mut class_j = 0;
        if num_classes > 0 {
          let cw = w / num_classes as f32;
          for class in classes.iter() {
            let count = class.count;
            let class_id = class.class_id;
            for _ in 0..count {
              let color = match class_id % 16 {
                0 => Color32::BLUE,
                1 => Color32::GREEN,
                2 => Color32::RED,
                3 => Color32::YELLOW,
                4 => Color32::LIGHT_BLUE,
                5 => Color32::LIGHT_GREEN,
                6 => Color32::LIGHT_RED,
                7 => Color32::KHAKI,
                8 => Color32::BLUE,
                9 => Color32::DARK_RED,
                10 => Color32::GOLD,
                11 => Color32::DARK_GREEN,
                12 => Color32::LIGHT_YELLOW,
                13 => Color32::DARK_BLUE,
                14 => Color32::BROWN,
                15 => Color32::DARK_GRAY,
                _ => unreachable!(),
              };
              let cx1 = x1 + cw * class_j as f32;
              let cx2 = cx1 + cw;
              let class_rect = Rect::from_x_y_ranges(cx1..=cx2, y1..=y2);
              painter.rect_filled(class_rect, Rounding::same(3.0), color);
              class_j += 1;
            }
          }
        } else {
          let color = Color32::DARK_GRAY;
          let timeslot_rect = Rect::from_x_y_ranges(x1..=x2, y1..=y2);
          painter.rect_stroke(timeslot_rect, Rounding::none(), Stroke::new(1.0, color));
        }
      }
    }

    response
  }
}
