use crate::{calendar_widget::CalendarWidget, thread_simulation::ThreadSimulation, evaluators};
use eframe::{egui, epi};

pub struct MyApp {
    simulation: ThreadSimulation,
    sim_run_steps: usize,
    sim_progress_report_interval: usize,
    show_class_editor: bool,
}

impl MyApp {
    pub fn new(simulation: ThreadSimulation) -> MyApp {
        Self { simulation, sim_run_steps: 0, sim_progress_report_interval: 1000, show_class_editor: false }
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::Window::new("Editor de clases").open(&mut self.show_class_editor).show(ctx, |ui| {
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                ui.label("Profe 1");
                ui.label("Profe 2");
                ui.label("Profe 3");
                ui.label("Profe 4");
                ui.label("Profe 5");
                ui.label("Profe 6");
            });
            ui.label("Hello World!");
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Clases", |ui| {
                    if ui.button("Editar").clicked() {
                        self.show_class_editor = true;
                    }
                })
            });
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
                {
                    let mut inner_simulation = self.simulation.simulation.lock().unwrap();
                    // TODO: Neater interface for aquiring inner struct

                    for evaluator in inner_simulation.get_evaluators_mut() {
                        ui.vertical(|ui| {
                            ui.label(evaluator.get_name());
                            for parameter in evaluator.get_parameters_mut() {
                                ui.horizontal(|ui| {
                                    ui.label(parameter.name);
                                    match parameter.value {
                                        evaluators::ParameterValue::F32(value) => {
                                            ui.add(egui::DragValue::new(value));
                                        },
                                        evaluators::ParameterValue::Usize(value) => {
                                            ui.add(egui::DragValue::new(value));
                                        }
                                    }
                                });
                            }
                        });
                    }
                }

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
