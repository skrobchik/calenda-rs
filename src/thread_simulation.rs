use crate::{calendars::CalendarState, simulation::Simulation};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
};
struct ProgressReport {
    state: CalendarState,
    energy: f32,
    step: usize,
    total_run_steps: usize,
}

pub struct ThreadSimulation {
    pub simulation: Arc<Mutex<Simulation>>,
    running: Arc<AtomicBool>,
    rx: Option<Receiver<ProgressReport>>,
    latest_progress_report: ProgressReport,
}

#[derive(Debug)]
pub struct SimulationRunningError;

impl ThreadSimulation {
    pub fn new(sim: Simulation) -> Self {
        let progress_report = ProgressReport {
            state: sim.get_current_state().clone(),
            energy: sim.get_current_energy(),
            step: 0,
            total_run_steps: 0,
        };
        ThreadSimulation {
            simulation: Arc::new(Mutex::new(sim)),
            running: Arc::new(AtomicBool::new(false)),
            rx: None,
            latest_progress_report: progress_report,
        }
    }

    fn receive_latest_progress_report(&mut self) {
        if self.rx.is_none() {
            return;
        }
        if let Some(progress_report) = self.rx.as_ref().unwrap().try_iter().last() {
            self.latest_progress_report = progress_report;
        }
    }

    pub fn get_job_step(&mut self) -> Option<usize> {
        if self.is_running() {
            self.receive_latest_progress_report();
            Some(self.latest_progress_report.step)
        } else {
            None
        }
    }

    pub fn get_latest_state(&mut self) -> CalendarState {
        if self.is_running() {
            self.receive_latest_progress_report();
            self.latest_progress_report.state.clone()
        } else {
            self.simulation.lock().unwrap().get_current_state().clone()
        }
    }

    pub fn get_latest_energy(&mut self) -> f32 {
        if self.is_running() {
            self.receive_latest_progress_report();
            self.latest_progress_report.energy
        } else {
            self.simulation.lock().unwrap().get_current_energy()
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn run_sim_job(
        &mut self,
        sim_job_steps: usize,
        progress_report_interval: usize,
    ) -> Result<(), SimulationRunningError> {
        if self.is_running() {
            return Err(SimulationRunningError{});
        }
        if sim_job_steps == 0 {
            return Ok(());
        }

        let (tx, rx) = mpsc::channel();
        let local_simulation = self.simulation.clone();
        let local_running_flag = self.running.clone();

        self.rx = Some(rx);
        self.running.store(true, Ordering::Relaxed);

        thread::spawn(move || {
            let mut sim = local_simulation.lock().unwrap();
            for step in 1..=sim_job_steps {
                if progress_report_interval != 0 && step % progress_report_interval == 0 {
                    tx.send(ProgressReport {
                        state: sim.get_current_state().clone(),
                        energy: sim.get_current_energy(),
                        step,
                        total_run_steps: sim_job_steps,
                    }).unwrap();
                }
                sim.step();
            }
            local_running_flag.store(false, Ordering::Relaxed);
        });

        Ok(())
    }
}
