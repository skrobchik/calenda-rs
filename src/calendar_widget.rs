use egui::{Widget, Vec2, Sense, Rounding, Color32, Rect, Stroke};

use crate::calendars::CalendarState;
use crate::timeslot;

pub struct CalendarWidget<'a> {
    state: &'a CalendarState
}

impl<'a> CalendarWidget<'a> {
    pub fn new(calendar_state: &CalendarState) -> CalendarWidget {
        CalendarWidget { state: calendar_state }
    }
}

impl Widget for CalendarWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let w = 100.0;
        let h = 20.0;
        let desired_size = Vec2::new(timeslot::DAY_COUNT as f32*w, timeslot::TIMESLOT_COUNT as f32*h);
        let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
        let rect = response.rect;
        
        for day in timeslot::DAY_RANGE {
            for timeslot in timeslot::TIMESLOT_RANGE {
                let x0 = rect.left_top().x;
                let y0 = rect.left_top().y;
                let x1 = x0 + w*(day as f32);
                let y1 = y0 + h*(timeslot as f32);
                let x2 = x1 + w;
                let y2 = y1 + h;
                let classes = &self.state.get_schedule_matrix()[day][timeslot];
                let num_classes = classes.count_total();
                let mut class_j = 0;
                if num_classes > 0 {
                    let cw = w / num_classes as f32;
                    for (class_id, count) in classes.iter() {
                        for _ in 0..*count {
                            let color = match class_id {
                                0 => Color32::BLUE,
                                1 => Color32::GREEN,
                                2 => Color32::RED,
                                3 => Color32::YELLOW,
                                4 => Color32::LIGHT_BLUE,
                                5 => Color32::LIGHT_GREEN,
                                6 => Color32::LIGHT_RED,
                                7 => Color32::KHAKI,
                                _ => Color32::BLACK
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