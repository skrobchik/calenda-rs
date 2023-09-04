use egui::ScrollArea;

use crate::school_schedule::SchoolSchedule;

pub(crate) struct ProfessorEditor<'a> {
  state: &'a mut SchoolSchedule,
  availability_editor_professor_id: &'a mut Option<usize>,
  availability_editor_widget_open: &'a mut bool,
}

impl<'a> ProfessorEditor<'a> {
  pub(crate) fn new(
    state: &'a mut SchoolSchedule,
    availability_editor_professor_id: &'a mut Option<usize>,
    availability_editor_widget_open: &'a mut bool,
  ) -> Self {
    ProfessorEditor {
      state,
      availability_editor_professor_id,
      availability_editor_widget_open,
    }
  }

  pub(crate) fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Professor Editor")
      .open(open)
      .resizable(true)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }

  pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
    ui.separator();
    let num_professors = self.state.get_num_professors();
    ScrollArea::vertical()
      .auto_shrink([false; 2])
      .max_height(500.0)
      .show(ui, |ui| {
        for professor_id in 0..num_professors {
          ui.horizontal(|ui| {
            ui.label("Nombre");
            ui.text_edit_singleline(&mut self.state.get_professor_metadata_mut(professor_id).unwrap().name);
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
