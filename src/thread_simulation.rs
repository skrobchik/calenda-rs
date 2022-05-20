use rand::{prelude::StdRng, SeedableRng};

use crate::{calendars::CalendarState, evaluators::Evaluator};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
};

use rand::Rng;

struct SimulationJob {
    current_job_step: usize,
    total_job_steps: usize,
    state: CalendarState,
    energy: f32,
    temperature_function: Box<dyn Fn(f32) -> f32 + Send>,
    acceptance_probability_function: Box<dyn Fn(f32, f32, f32) -> f32 + Send>,
    evaluators: Vec<Evaluator>,
    rng: StdRng,
}

impl SimulationJob {
    pub fn new(state: CalendarState, total_job_steps: usize, evaluators: Vec<Evaluator>) -> Self {
        let mut s = SimulationJob {
            current_job_step: 0,
            total_job_steps,
            state,
            energy: f32::INFINITY,
            temperature_function: Box::new(default_temperature_function),
            acceptance_probability_function: Box::new(default_acceptance_probability_function),
            evaluators,
            rng: StdRng::from_entropy(),
        };
        s.energy = s.calculate_energy(&s.state);
        s
    }
    fn calculate_energy(&self, state: &CalendarState) -> f32 {
        let mut energy: f32 = 0.0;
        for evaluator in &self.evaluators {
            energy += evaluator.evaluate(&state);
        }
        energy
    }
    pub fn step(&mut self) {
        let new_state = self.state.get_random_neighbor(&mut self.rng).unwrap();
        let new_energy = self.calculate_energy(&new_state);
        let progress_ratio = (self.current_job_step as f32) / (self.total_job_steps as f32);
        let temperature = (self.temperature_function)(progress_ratio);
        let p = (self.acceptance_probability_function)(self.energy, new_energy, temperature);
        if self.rng.gen_bool(p.into()) {
            self.state = new_state;
            self.energy = new_energy;
        }
        self.current_job_step += 1;
    }
}

fn default_temperature_function(x: f32) -> f32 {
    (1.0 / (x + 1.0)) - 0.5 * ((2.0 * std::f32::consts::PI * x).cos()).powi(2)
}

fn default_acceptance_probability_function(
    current_energy: f32,
    new_energy: f32,
    temperature: f32,
) -> f32 {
    if new_energy < current_energy {
        1.0
    } else {
        (-(new_energy - current_energy) / temperature).exp()
    }
}

pub struct ThreadSimulation {
    pub state: CalendarState,
    pub evaluators: Vec<Evaluator>,
    running: Arc<AtomicBool>,
    rx: Option<Receiver<usize>>,
    job_join_handle: Option<JoinHandle<CalendarState>>,
}

#[derive(Debug)]
pub struct SimulationRunningError;

impl ThreadSimulation {
    pub fn new() -> Self {
        ThreadSimulation {
            state: CalendarState::new(),
            running: Arc::new(AtomicBool::new(false)),
            rx: None,
            evaluators: Vec::new(),
            job_join_handle: None,
        }
    }

    /*
    fn receive_latest_progress_report(&mut self) {
        if self.rx.is_none() {
            return;
        }
        if let Some(progress_report) = self.rx.as_ref().unwrap().try_iter().last() {
            //self.latest_progress_report = progress_report;
        }
    }
    */

    /*pub fn get_job_step(&mut self) -> Option<usize> {
        if self.is_running() {
            self.receive_latest_progress_report();
            Some(self.latest_progress_report.step)
        } else {
            None
        }
    }*/

    pub fn is_job_running(&mut self) -> bool {
        let is_running = self.running.load(Ordering::Relaxed);
        if !is_running {
            if let Some(handle) = self.job_join_handle.take() {
                self.state = handle.join().unwrap();
            }
        }
        is_running
    }

    pub fn run_sim_job(
        &mut self,
        sim_job_steps: usize,
        progress_report_interval: usize,
    ) -> Result<(), SimulationRunningError> {
        if self.is_job_running() {
            return Err(SimulationRunningError {});
        }
        if sim_job_steps == 0 {
            return Ok(());
        }

        let (tx, rx) = mpsc::channel();

        self.rx = Some(rx);

        let evaluators_copy = self.evaluators.clone();
        let mut sim_job = SimulationJob::new(self.state.clone(), sim_job_steps, evaluators_copy);

        let local_running_flag = self.running.clone();
        self.running.store(true, Ordering::Relaxed);
        self.job_join_handle = Some(thread::spawn(move || {
            for step in 1..=sim_job_steps {
                /*
                if progress_report_interval != 0 && step % progress_report_interval == 0 {
                    tx.send(ProgressReport {
                        state: sim.get_current_state().clone(),
                        energy: sim.get_current_energy(),
                        step,
                        total_run_steps: sim_job_steps,
                    })
                    .unwrap();
                }*/
                sim_job.step();
            }
            local_running_flag.store(false, Ordering::Relaxed);
            return sim_job.state;
        }));

        Ok(())
    }
}
