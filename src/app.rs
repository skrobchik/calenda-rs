use crate::{calendar_widget::CalendarWidget, thread_simulation::ThreadSimulation};
use eframe::{egui, epi};

pub struct MyApp {
    simulation: ThreadSimulation,
    sim_run_steps: usize,
    sim_progress_report_interval: usize
}

impl MyApp {
    pub fn new(simulation: ThreadSimulation) -> MyApp {
        Self { simulation, sim_run_steps: 0, sim_progress_report_interval: 1000 }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.add(CalendarWidget::new(&self.simulation.get_latest_state()));
            ui.label(format!(
                "Energy: {}",
                self.simulation.get_latest_energy()
            ));
            ui.horizontal(|ui| {
                ui.label("Steps:");
                ui.add(egui::DragValue::new(&mut self.sim_run_steps));
            });
            ui.horizontal(|ui| {
                ui.label("Progress Report Inteval:");
                ui.add(egui::DragValue::new(&mut self.sim_progress_report_interval));
            });
            if !self.simulation.is_running() {
                if ui.button(format!("Run Simulation for {} steps", self.sim_run_steps)).clicked(){
                    self.simulation.run_sim_job(self.sim_run_steps, self.sim_progress_report_interval).unwrap();
                }
            } else {
                ui.label(format!("Simulation running... ({}/{})", self.simulation.get_job_step().unwrap(), self.sim_run_steps));
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}
