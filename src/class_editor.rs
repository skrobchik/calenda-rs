use egui::{ComboBox, ScrollArea};

use crate::school_schedule::{ClassroomType, SchoolSchedule};

pub struct ClassEditor<'a> {
  state: &'a mut SchoolSchedule,
}

impl<'a> ClassEditor<'a> {
  pub fn new(state: &'a mut SchoolSchedule) -> Self {
    ClassEditor { state }
  }

  pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Class Editor")
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
    let (mut classes, professors) = self.state.get_classes_and_professors_mut();
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
            ComboBox::from_id_source(*class_id)
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
          ui.horizontal(|ui| {
            ui.label("Profesor");
            ui.label(format!("{}", class.professor));
            ComboBox::from_id_source(format!("professor_selector_{}", class_id.clone()))
              .selected_text(
                professors
                  .iter()
                  .find(|(_, _, professor_id)| *professor_id == class.professor)
                  .map_or("Undefined Professor", |(_, metadata, _)| &metadata.name),
              )
              .show_ui(ui, |ui| {
                for (_professor, metadata, professor_id) in professors.iter() {
                  ui.selectable_value(&mut class.professor, *professor_id, &metadata.name);
                }
              })
          });
          ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut class.class_hours, 0..=30).text("Tiempo (x30 min)"));
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
