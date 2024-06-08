use std::{
  collections::BTreeMap,
  time::{Duration, Instant},
};

use super::methods::simulated_annealing::StopCondition;

#[derive(thiserror::Error, Debug)]
pub enum StatsTrackerError {
  #[error("You are logging the same stat twice per step `{0}`")]
  MultiStatLogging(String),
  #[error("You missed logging this stat in a step `{0}`")]
  MissedStatLogging(String),
}

pub enum SamplingRate {
  Steps(usize),
  Duration(Duration),
}

impl From<usize> for SamplingRate {
  fn from(value: usize) -> Self {
    SamplingRate::Steps(value)
  }
}

impl From<Duration> for SamplingRate {
  fn from(value: Duration) -> Self {
    SamplingRate::Duration(value)
  }
}

pub struct StatsTracker {
  step_index: usize,
  stats_index: usize,
  sampling_rate: SamplingRate,
  stats: BTreeMap<String, Vec<serde_json::Value>>,
  is_logging_step: bool,
  latest_logging_step_start_instant: Instant,
}

impl StatsTracker {
  pub fn new<T: Into<SamplingRate>>(sampling_rate: T) -> Self {
    StatsTracker {
      step_index: 0,
      stats_index: 0,
      sampling_rate: sampling_rate.into(),
      stats: Default::default(),
      is_logging_step: true,
      latest_logging_step_start_instant: Instant::now(),
    }
  }

  pub fn with_estimated_size(total_runtime: &StopCondition, estimated_size: usize) -> Self {
    let sampling_rate = match total_runtime {
      StopCondition::Steps(steps) => SamplingRate::from(steps.div_ceil(estimated_size)),
      StopCondition::Time(time) => SamplingRate::from(time.div_f64(estimated_size as f64)),
    };
    Self::new(sampling_rate)
  }

  pub fn into_stats(self) -> BTreeMap<String, Vec<serde_json::Value>> {
    self.stats
  }

  pub fn inc_step(&mut self) {
    self.step_index += 1;
    if match self.sampling_rate {
      SamplingRate::Steps(sampling_rate) => self.step_index % sampling_rate == 0,
      SamplingRate::Duration(sampling_rate) => {
        self.latest_logging_step_start_instant.elapsed() > sampling_rate
      }
    } {
      self.is_logging_step = true;
      self.stats_index += 1;
      self.latest_logging_step_start_instant = Instant::now();
    } else {
      self.is_logging_step = false;
    }
  }

  pub fn log_stat<T: Into<serde_json::Value>>(
    &mut self,
    label: &str,
    value: T,
  ) -> Result<(), StatsTrackerError> {
    if !self.is_logging_step {
      return Ok(());
    }
    let value: serde_json::Value = value.into();
    let stat_vector = self.stats.entry(label.into()).or_default();
    match stat_vector.len().cmp(&self.stats_index) {
      std::cmp::Ordering::Less => Err(StatsTrackerError::MissedStatLogging(label.into())),
      std::cmp::Ordering::Equal => {
        stat_vector.push(value);
        Ok(())
      }
      std::cmp::Ordering::Greater => Err(StatsTrackerError::MultiStatLogging(label.into())),
    }
  }
}
