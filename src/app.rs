use crate::{
  calendar_widget::CalendarWidget,
  evaluators,
  metadata_register::{
    ClassMetadata, ClassRoomType, MetadataRegister, ProfessorMetadata, SemesterNumber,
  },
  thread_simulation::ThreadSimulation,
};
use eframe::{egui};
use egui::{Context, TextStyle};
use serde::{Deserialize, Serialize};

use evaluators::Evaluator;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MyApp {
  simulation: ThreadSimulation,
  is_simulation_running: bool,
  sim_run_steps: usize,
  sim_progress_report_interval: usize,
  show_professor_editor: bool,
  show_class_time_editor: bool,
  show_simulation_parameter_editor: bool,
  metadata_register: MetadataRegister,
  selected_class: Option<usize>,
}

impl MyApp {
  pub fn new(cc: &eframe::CreationContext<'_>) -> MyApp {
    if let Some(storage) = cc.storage {
      return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
    }
    Default::default()
  }
  fn draw_professor_editor(&mut self, ctx: &Context) {
    egui::SidePanel::right("professor_edit_panel").show(ctx, |ui| {
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
    egui::SidePanel::right("parameter_edit_panel").show(ctx, |ui| {
      ui.add_enabled_ui(!self.is_simulation_running, |ui| {
        ui.label("Parametros de Simulacion");
        ui.separator();
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
              ui.separator();
            }
          })
      });
    });
  }
  fn draw_class_time_editor(&mut self, ctx: &Context) {
    egui::SidePanel::right("class_time_editor").show(ctx, |ui| {
      ui.add_enabled_ui(!self.is_simulation_running, |ui| {
        ui.label("Tipos de Clases");
        ui.separator();

        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        let total_rows = self.metadata_register.class_register_len();
        egui::ScrollArea::vertical()
          .auto_shrink([false, false])
          .max_height(ui.available_height() - 5.0 * row_height)
          .stick_to_bottom()
          .show_rows(ui, row_height, total_rows, |ui, row_range| {
            for row in row_range {
              let class = self.metadata_register.get_class_metadata(row).unwrap();
              ui.horizontal(|ui| {
                ui.label(format!("C{}", row));
                if ui.button(class.name.clone()).clicked() {
                  self.selected_class = Some(row);
                }
                if ui.button("-").clicked() {
                  self.simulation.state.decrement_class_time(row);
                }
                ui.label(format!("{}", self.simulation.state.count_class_time(row)));
                if ui.button("+").clicked() {
                  self.simulation.state.increment_class_time(row);
                }
              });
            }
          });
        ui.separator();
        if ui.button("Nuevo").clicked() {
          self.metadata_register.add_class(ClassMetadata {
            name: "Nueva Clase".to_string(),
            classroom_type: ClassRoomType::SmallClassroom,
            semester_number: SemesterNumber::S1,
            professor_id: 0,
          });
        }
      });
    });
  }
  fn draw_class_editor(&mut self, ctx: &Context, selected_class: usize) {
    let professors: Vec<ProfessorMetadata> = self.metadata_register.get_professor_list().clone();
    let class = match self
      .metadata_register
      .get_class_metadata_mut(selected_class)
    {
      Some(c) => c,
      None => return,
    };
    egui::SidePanel::right("class_editor").show(ctx, |ui| {
      ui.add_enabled_ui(!self.is_simulation_running, |ui| {
        ui.label("Editor de Clase");
        ui.separator();
        ui.horizontal(|ui| {
          ui.label("Nombre");
          ui.text_edit_singleline(&mut class.name);
        });
        ui.horizontal(|ui| {
          egui::ComboBox::from_label("Profesor")
            .selected_text(&professors[class.professor_id].name)
            .show_ui(ui, |ui| {
              for (i, prof) in professors.iter().enumerate() {
                ui.selectable_value(&mut class.professor_id, i, &prof.name);
              }
            });
        });
      })
    });
  }
}

impl eframe::App for MyApp {
  fn save(&mut self, storage: &mut dyn eframe::Storage) {
    eframe::set_value(storage, eframe::APP_KEY, self);
  }

  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    self.is_simulation_running = self.simulation.is_job_running();

    if self.show_professor_editor {
      self.draw_professor_editor(ctx);
    }
    if self.show_simulation_parameter_editor {
      self.draw_parameter_editor(ctx);
    }
    if self.show_class_time_editor {
      self.draw_class_time_editor(ctx);
    }
    if let Some(selected_class) = self.selected_class {
      self.draw_class_editor(ctx, selected_class);
    }

    egui::CentralPanel::default().show(ctx, |ui| {
      egui::menu::bar(ui, |ui| {
        ui.menu_button("Vista", |ui| {
          if ui.button("Editor de Profesores").clicked() {
            self.show_professor_editor = !self.show_professor_editor;
          }
          if ui.button("Editor de Parametros de Simulacion").clicked() {
            self.show_simulation_parameter_editor = !self.show_simulation_parameter_editor;
          }
          if ui.button("Editor de Clases").clicked() {
            self.show_class_time_editor = !self.show_class_time_editor;
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
      ui.add_enabled_ui(!self.is_simulation_running, |ui| {
        if ui
          .button(format!("Run Simulation for {} steps", self.sim_run_steps))
          .clicked()
        {
          self
            .simulation
            .run_sim_job(self.sim_run_steps, self.sim_progress_report_interval)
            .unwrap();
        }
      })
    });

    // Resize the native window to be just the size we need it to be:
    frame.set_window_size(ctx.used_size());
  }
}

impl Default for MyApp {
  fn default() -> Self {
    let mut thread_simulation = ThreadSimulation::new();
    thread_simulation.evaluators = vec![
      Evaluator::GapCount { weight: 0.0 },
      Evaluator::Daylight {
        weight: 0.0,
        wake_up_time: 1,
        sleep_time: 8,
      },
      Evaluator::Colliding { weight: 0.0 },
      Evaluator::DailyWorkDifference { weight: 0.0 },
      Evaluator::SessionLengthLimits {
        weight: 0.0,
        min_len: 2,
        max_len: 4,
      },
    ];
    Self {
      simulation: thread_simulation,
      is_simulation_running: Default::default(),
      sim_run_steps: Default::default(),
      sim_progress_report_interval: Default::default(),
      show_professor_editor: Default::default(),
      show_class_time_editor: Default::default(),
      show_simulation_parameter_editor: Default::default(),
      metadata_register: Default::default(),
      selected_class: Default::default(),
    }
  }
}
