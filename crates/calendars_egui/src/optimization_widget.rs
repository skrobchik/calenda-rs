use std::time::Duration;

use calendars_core::StopCondition;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptimizationWidget {
  pub open: bool,
  current_stop_condition: StopCondition,
}

impl Default for OptimizationWidget {
  fn default() -> Self {
    Self {
      open: true,
      current_stop_condition: Default::default(),
    }
  }
}

impl OptimizationWidget {
  pub fn show(
    &mut self,
    ctx: &egui::Context,
    pb: Option<&indicatif::ProgressBar>,
  ) -> Option<StopCondition> {
    let mut open = self.open;
    let mut result: Option<StopCondition> = None;
    egui::Window::new("Optimizador de horario")
      .open(&mut open)
      .resizable(true)
      .show(ctx, |ui| {
        result = self.ui(ui, pb);
      });
    self.open = open;
    result
  }

  fn ui(
    &mut self,
    ui: &mut egui::Ui,
    pb: Option<&indicatif::ProgressBar>,
  ) -> Option<StopCondition> {
    if ui
      .add(egui::RadioButton::new(
        matches!(self.current_stop_condition, StopCondition::Steps(_)),
        "Pasos de simulacion",
      ))
      .clicked()
    {
      self.current_stop_condition = StopCondition::Steps(0);
    };
    if ui
      .add(egui::RadioButton::new(
        matches!(self.current_stop_condition, StopCondition::Time(_)),
        "Tiempo de simulacion",
      ))
      .clicked()
    {
      self.current_stop_condition = StopCondition::Time(Duration::ZERO);
    }
    match &mut self.current_stop_condition {
      StopCondition::Steps(n) => {
        ui.add(egui::widgets::DragValue::new(n));
      }
      StopCondition::Time(d) => {
        let mut n = d.as_secs();
        ui.add(egui::widgets::DragValue::new(&mut n));
        ui.label("segundos");
        *d = Duration::from_secs(n);
      }
    };
    if let Some(pb) = pb {
      let l = pb.length().unwrap();
      let i = pb.position();
      let p = (i as f32) / (l as f32);
      let pb = egui::ProgressBar::new(p).show_percentage();
      ui.add(pb);
      None
    } else if ui.button("Optimizar").clicked() {
      Some(self.current_stop_condition.clone())
    } else {
      None
    }
  }
}
