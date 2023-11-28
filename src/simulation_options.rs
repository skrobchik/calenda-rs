use serde::{Deserialize, Serialize};

use crate::school_schedule::{class_calendar::ClassCalendar, SimulationConstraints};

#[derive(Debug, Clone, Default)]
pub(crate) enum ProgressOption {
  ProgressBar(indicatif::ProgressBar),
  MultiProgress(indicatif::MultiProgress),
  #[default]
  None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum TemperatureFunction {
  T001,
  T002,
  T003,
  T004,
}

const _: () = {
  fn assert_send<T: Send>() {}
  fn assert_sync<T: Sync>() {}
  fn assert_all() {
    assert_send::<SimulationOptions>();
    assert_sync::<SimulationOptions>();
  }
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct AdvancedSimulationOptions {
  pub(crate) progress_bar_update_interval: usize,
}

impl Default for AdvancedSimulationOptions {
  fn default() -> Self {
    Self {
      progress_bar_update_interval: 100,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SimulationOptions {
  pub(crate) simulation_constraints: SimulationConstraints,
  pub(crate) total_steps: usize,
  pub(crate) initial_state: Option<ClassCalendar>,
  #[serde(skip)]
  pub(crate) progress: ProgressOption,
  pub(crate) temperature_function: TemperatureFunction,
  pub(crate) advanced_options: AdvancedSimulationOptions,
}
