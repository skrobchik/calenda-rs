use egui::Color32;

use crate::school_schedule::{Availability, Professor};
use crate::timeslot;

pub struct ProfessorScheduleWidget<'a> {
  state: &'a mut Professor,
}

impl<'a> ProfessorScheduleWidget<'a> {
  pub fn new(state: &'a mut Professor) -> ProfessorScheduleWidget<'a> {
    ProfessorScheduleWidget { state }
  }
  pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Professor")
      .open(open)
      .vscroll(false)
      .resizable(true)
      .default_height(500.0)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }
  fn ui(&mut self, ui: &mut egui::Ui) {
    egui::Grid::new("my_grid").show(ui, |ui| {
      for j in timeslot::TIMESLOT_RANGE {
        for i in timeslot::DAY_RANGE {
          let av = &mut self.state.availability[i][j];
          let text = match av {
            Availability::Available => "1",
            Availability::AvailableIfNeeded => "2",
            Availability::NotAvailable => "3",
          };
          let color = match av {
            Availability::Available => Color32::GREEN,
            Availability::AvailableIfNeeded => Color32::YELLOW,
            Availability::NotAvailable => Color32::LIGHT_RED,
          };
          if ui.add(egui::Button::new(text).fill(color)).clicked() {
            *av = match av {
              Availability::Available => Availability::NotAvailable,
              Availability::AvailableIfNeeded => Availability::Available,
              Availability::NotAvailable => Availability::AvailableIfNeeded,
            };
          }
        }
        ui.end_row();
      }
    });
  }
}
