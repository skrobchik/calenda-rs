use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum StatsTrackerError {
  #[error("You are logging the same stat twice per step `{0}`")]
  MultiStatLogging(String),
  #[error("You missed logging this stat in a step `{0}`")]
  MissedStatLogging(String),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct StatsTracker {
  step_index: usize,
  stats_index: usize,
  sampling_rate: usize,
  stats: BTreeMap<String, Vec<serde_json::Value>>,
  is_logging_step: bool,
}

impl StatsTracker {
  pub(crate) fn new(sampling_rate: usize) -> Self {
    StatsTracker {
      step_index: 0,
      stats_index: 0,
      sampling_rate,
      stats: Default::default(),
      is_logging_step: true,
    }
  }

  pub(crate) fn into_stats(self) -> BTreeMap<String, Vec<serde_json::Value>> {
    self.stats
  }

  pub(crate) fn inc_step(&mut self) {
    self.step_index += 1;
    if self.step_index % self.sampling_rate == 0 {
      self.is_logging_step = true;
      self.stats_index += 1;
    } else {
      self.is_logging_step = false;
    }
  }

  pub(crate) fn log_stat<T: Into<serde_json::Value>>(
    &mut self,
    stat_label: &str,
    stat_value: T,
  ) -> Result<(), StatsTrackerError> {
    if !self.is_logging_step {
      return Ok(());
    }
    let stat_value: serde_json::Value = stat_value.into();
    let stat_vector = self.stats.entry(stat_label.into()).or_default();
    match stat_vector.len().cmp(&self.stats_index) {
      std::cmp::Ordering::Less => Err(StatsTrackerError::MissedStatLogging(stat_label.into())),
      std::cmp::Ordering::Equal => {
        stat_vector.push(stat_value);
        Ok(())
      }
      std::cmp::Ordering::Greater => Err(StatsTrackerError::MultiStatLogging(stat_label.into())),
    }
  }
}
