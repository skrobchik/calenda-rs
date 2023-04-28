use egui::{ComboBox, ScrollArea};

use crate::school_schedule::{ClassroomType, SchoolSchedule};

pub struct ProfessorEditor<'a> {
  state: &'a mut SchoolSchedule,
  availability_editor_professor_id: &'a mut Option<usize>,
  availability_editor_widget_open: &'a mut bool
}

impl<'a> ProfessorEditor<'a> {
  pub fn new(state: &'a mut SchoolSchedule, availability_editor_professor_id: &'a mut Option<usize>, availability_editor_widget_open: &'a mut bool) -> Self {
    ProfessorEditor { state, availability_editor_professor_id, availability_editor_widget_open }
  }

  pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Professor Editor")
      .open(open)
      .vscroll(false)
      .resizable(true)
      .default_height(500.0)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) {
    ui.separator();
    let mut professors = self.state.get_professors_mut();
    let text_style = egui::TextStyle::Body;
    ScrollArea::new([false, true]).show_rows(
      ui,
      ui.text_style_height(&text_style),
      professors.len(),
      |ui, row_range| {
        let class_range = professors.get_mut(row_range).unwrap();
        for (professor, metadata, professor_id) in class_range.iter_mut() {
          ui.horizontal(|ui| {
            ui.label("Nombre");
            ui.text_edit_singleline(&mut metadata.name);
          });
          if ui.button("Editar disponibilidad").clicked() {
            *self.availability_editor_professor_id = Some(*professor_id);
            *self.availability_editor_widget_open = true;
          }
          ui.separator();
        }
      },
    );
    if ui.button("+").clicked() {
      self.state.add_new_professor();
    }
    self.state.fill_classes();
  }
}
