use calendars_core::{ProfessorKey, SchoolSchedule};
use egui::ScrollArea;

pub struct ProfessorEditor<'a> {
  state: &'a mut SchoolSchedule,
  availability_editor_professor_id: &'a mut Option<ProfessorKey>,
  availability_editor_widget_open: &'a mut bool,
}

impl<'a> ProfessorEditor<'a> {
  pub fn new(
    state: &'a mut SchoolSchedule,
    availability_editor_professor_id: &'a mut Option<ProfessorKey>,
    availability_editor_widget_open: &'a mut bool,
  ) -> Self {
    ProfessorEditor {
      state,
      availability_editor_professor_id,
      availability_editor_widget_open,
    }
  }

  pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Editor de profesores")
      .open(open)
      .resizable(true)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) {
    ui.separator();
    ScrollArea::vertical()
      .auto_shrink([false; 2])
      .max_height(500.0)
      .show(ui, |ui| {
        let professor_keys: Vec<ProfessorKey> = self
          .state
          .get_simulation_constraints()
          .professors.keys()
          .collect();
        for professor_id in professor_keys {
          ui.horizontal(|ui| {
            ui.label("Nombre");
            ui.text_edit_singleline(
              &mut self
                .state
                .get_professor_metadata_mut(professor_id)
                .unwrap()
                .name,
            );
          });
          if ui.button("Editar disponibilidad").clicked() {
            *self.availability_editor_professor_id = Some(professor_id);
            *self.availability_editor_widget_open = true;
          }
          ui.separator();
        }
      });
    if ui.button("+").clicked() {
      self.state.add_new_professor();
    }
  }
}
