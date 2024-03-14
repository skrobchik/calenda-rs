use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::school_schedule::{class_calendar::ClassCalendar, SimulationConstraints};

#[derive(Debug, Clone, Default)]
pub enum ProgressOption {
  ProgressBar(indicatif::ProgressBar),
  MultiProgress(indicatif::MultiProgress),
  #[default]
  None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TemperatureFunction {
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
pub struct AdvancedSimulationOptions {
  pub progress_bar_update_interval: usize,
  #[serde(skip)]
  pub live_update: Option<LiveUpdate>,
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
pub enum StopCondition {
  Steps(usize),
  Time(Duration),
}

impl Default for StopCondition {
  fn default() -> Self {
    StopCondition::Steps(0)
  }
}

#[derive(Debug, Clone)]
pub struct LiveUpdate {
  pub channel: std::sync::mpsc::Sender<ClassCalendar>,
  pub live_update_interval: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimulationOptions {
  pub simulation_constraints: SimulationConstraints,
  pub stop_condition: StopCondition,
  pub initial_state: Option<ClassCalendar>,
  #[serde(skip)]
  pub progress: ProgressOption,
  pub temperature_function: TemperatureFunction,
  pub advanced_options: AdvancedSimulationOptions,
}
