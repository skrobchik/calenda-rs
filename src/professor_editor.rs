use egui::{ComboBox, ScrollArea};

use crate::school_schedule::{ClassroomType, SchoolSchedule};

pub struct ProfessorEditor<'a> {
  state: &'a mut SchoolSchedule,
}

impl<'a> ProfessorEditor<'a> {
  pub fn new(state: &'a mut SchoolSchedule) -> Self {
    ProfessorEditor { state }
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
    let mut classes = self.state.get_classes_mut();
    let text_style = egui::TextStyle::Body;
    ScrollArea::new([false, true]).show_rows(
      ui,
      ui.text_style_height(&text_style),
      classes.len(),
      |ui, row_range| {
        let class_range = classes.get_mut(row_range).unwrap();
        for (class, metadata, class_id) in class_range.iter_mut() {
          ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut metadata.color);
            ui.text_edit_singleline(&mut metadata.name);
          });
          ui.horizontal(|ui| {
            ui.label("Aula");
            ComboBox::from_id_source(class_id)
              .selected_text(class.classroom_type.to_string())
              .show_ui(ui, |ui| {
                for classroom_type_variant in enum_iterator::all::<ClassroomType>() {
                  ui.selectable_value(
                    &mut class.classroom_type,
                    classroom_type_variant,
                    classroom_type_variant.to_string(),
                  );
                }
              });
          });
          ui.separator();
        }
      },
    );
    if ui.button("+").clicked() {
      self.state.add_new_class();
    }
    self.state.fill_classes();
  }
}
