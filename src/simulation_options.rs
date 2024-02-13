use std::time::Duration;

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
  Linear,
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
  #[serde(skip)]
  pub(crate) live_update: Option<LiveUpdate>,
}

impl Default for AdvancedSimulationOptions {
  fn default() -> Self {
    Self {
      progress_bar_update_interval: 100,
      live_update: None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum StopCondition {
  Steps(usize),
  Time(Duration),
}

impl Default for StopCondition {
  fn default() -> Self {
    StopCondition::Steps(0)
  }
}

#[derive(Debug, Clone)]
pub(crate) struct LiveUpdate {
  pub(crate) channel: std::sync::mpsc::Sender<ClassCalendar>,
  pub(crate) live_update_interval: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SimulationOptions {
  pub(crate) simulation_constraints: SimulationConstraints,
  pub(crate) stop_condition: StopCondition,
  pub(crate) initial_state: Option<ClassCalendar>,
  #[serde(skip)]
  pub(crate) progress: ProgressOption,
  pub(crate) temperature_function: TemperatureFunction,
  pub(crate) advanced_options: AdvancedSimulationOptions,
}
