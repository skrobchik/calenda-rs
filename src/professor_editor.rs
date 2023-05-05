use egui::ScrollArea;

use crate::school_schedule::SchoolSchedule;

pub struct ProfessorEditor<'a> {
  state: &'a mut SchoolSchedule,
  availability_editor_professor_id: &'a mut Option<usize>,
  availability_editor_widget_open: &'a mut bool,
}

impl<'a> ProfessorEditor<'a> {
  pub fn new(
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

  pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Professor Editor")
      .open(open)
      .resizable(true)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) {
    ui.separator();
    let mut professors = self.state.get_professors_mut();
    let text_style = egui::TextStyle::Body;
    let row_height = 2.0 * ui.spacing().interact_size.y + ui.text_style_height(&text_style);
    let num_rows = professors.len();
    ScrollArea::vertical().auto_shrink([false; 2]).max_height(500.0).show_rows(
      ui,
      row_height,
      num_rows,
      |ui, row_range| {
        let class_range = professors.get_mut(row_range).unwrap();
        for (_professor, metadata, professor_id) in class_range.iter_mut() {
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
