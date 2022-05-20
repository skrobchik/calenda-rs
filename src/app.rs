use crate::{
    calendar_widget::CalendarWidget,
    evaluators,
    metadata_register::{MetadataRegister, ProfessorMetadata},
    thread_simulation::ThreadSimulation,
};
use eframe::{egui, epi};
use egui::{Context, TextStyle};

pub struct MyApp {
    simulation: ThreadSimulation,
    is_simulation_running: bool,
    sim_run_steps: usize,
    sim_progress_report_interval: usize,
    show_class_editor: bool,
    show_simulation_parameter_editor: bool,
    metadata_register: MetadataRegister,
}

impl MyApp {
    pub fn new(simulation: ThreadSimulation) -> MyApp {
        let mut metadata_register = MetadataRegister::default();
        metadata_register.add_professor(ProfessorMetadata {
            name: "Jose Carlos".to_string(),
        });
        metadata_register.add_professor(ProfessorMetadata {
            name: "Ochoa".to_string(),
        });
        Self {
            simulation,
            is_simulation_running: false,
            sim_run_steps: 0,
            sim_progress_report_interval: 1000,
            show_class_editor: false,
            show_simulation_parameter_editor: false,
            metadata_register,
        }
    }
    fn draw_professor_editor(&mut self, ctx: &Context) {
        egui::SidePanel::right("Editor de clases").show(ctx, |ui| {
            ui.add_enabled_ui(!self.is_simulation_running, |ui| {
                ui.label("Profesores");
                ui.separator();
                let text_style = TextStyle::Body;
                let row_height = ui.text_style_height(&text_style);
                let total_rows = self.metadata_register.professor_register_len();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 5.0 * row_height)
                    .stick_to_bottom()
                    .show_rows(ui, row_height, total_rows, |ui, row_range| {
                        for row in row_range {
                            ui.horizontal(|ui| {
                                ui.label(format!("P{}", row));
                                let metadata = self
                                    .metadata_register
                                    .get_professor_metadata_mut(row)
                                    .unwrap();
                                ui.text_edit_singleline(&mut metadata.name);
                            });
                        }
                    });
                ui.separator();
                if ui.button("Nuevo").clicked() {
                    self.metadata_register.add_professor(ProfessorMetadata {
                        name: "Nuevo Profesor".to_string(),
                    });
                }
            });
        });
    }
    fn draw_parameter_editor(&mut self, ctx: &Context) {
        egui::SidePanel::right("Parametros de Simulacion").show(ctx, |ui| {
            ui.add_enabled_ui(!self.is_simulation_running, |ui| {
                let evaluators = &mut self.simulation.evaluators;
                let text_style = TextStyle::Body;
                let row_height = ui.text_style_height(&text_style);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, evaluators.len(), |ui, row_range| {
                        for row in row_range {
                            let evaluator = &mut evaluators[row];
                            ui.vertical(|ui| {
                                ui.label(evaluator.get_name());
                                for parameter in evaluator.get_parameters_mut() {
                                    ui.horizontal(|ui| {
                                        ui.label(parameter.name);
                                        match parameter.value {
                                            evaluators::ParameterValue::F32(value) => {
                                                ui.add(egui::DragValue::new(value));
                                            }
                                            evaluators::ParameterValue::Usize(value) => {
                                                ui.add(egui::DragValue::new(value));
                                            }
                                        }
                                    });
                                }
                            });
                        }
                    })
            });
        });
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        self.is_simulation_running = self.simulation.is_job_running();

        if self.show_class_editor {
            self.draw_professor_editor(ctx);
        }
        if self.show_simulation_parameter_editor {
            self.draw_parameter_editor(ctx);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Vista", |ui| {
                    if ui.button("Editor de Profesores").clicked() {
                        self.show_class_editor = !self.show_class_editor;
                    }
                    if ui.button("Editor de Parametros de Simulacion").clicked() {
                        self.show_simulation_parameter_editor =
                            !self.show_simulation_parameter_editor;
                    }
                })
            });
            ui.heading("My egui Application");
            ui.add(CalendarWidget::new(&self.simulation.state, 30.0, 10.0));
            ui.horizontal(|ui| {
                ui.label("Steps:");
                ui.add(egui::DragValue::new(&mut self.sim_run_steps));
            });
            ui.horizontal(|ui| {
                ui.label("Progress Report Inteval:");
                ui.add(egui::DragValue::new(&mut self.sim_progress_report_interval));
            });
            if self.is_simulation_running {
                {

                    //MyApp::draw_parameter_editor(&mut inner_simulation, ui);
                }

                if ui
                    .button(format!("Run Simulation for {} steps", self.sim_run_steps))
                    .clicked()
                {
                    self.simulation
                        .run_sim_job(self.sim_run_steps, self.sim_progress_report_interval)
                        .unwrap();
                }
            } else {
                /*ui.label(format!(
                    "Simulation running... ({}/{})",
                    self.simulation.get_job_step().unwrap(),
                    self.sim_run_steps
                ));
                */
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}
