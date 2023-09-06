use egui::{ComboBox, ScrollArea};

use crate::school_schedule::SchoolSchedule;

pub(crate) struct ClassEditor<'a> {
  state: &'a mut SchoolSchedule,
}

impl<'a> ClassEditor<'a> {
  pub(crate) fn new(state: &'a mut SchoolSchedule) -> Self {
    ClassEditor { state }
  }

  pub(crate) fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("Class Editor")
      .open(open)
      .resizable(true)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }

  pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
    ui.separator();
    let num_classes = self.state.get_num_classes();
    ScrollArea::vertical()
      .auto_shrink([false; 2])
      .max_height(500.0)
      .show(ui, |ui| {
        for class_id in 0..num_classes {
          ui.horizontal(|ui| {
            ui.color_edit_button_srgba(
              &mut self.state.get_class_metadata_mut(class_id).unwrap().color,
            );
            ui.text_edit_singleline(&mut self.state.get_class_metadata_mut(class_id).unwrap().name);
          });

          // TODO: Fix
          // ui.horizontal(|ui| {
          //   ui.label("Aula");
          //   ComboBox::from_id_source(format!("classroom_type_selector_{}", class_id))
          //     .selected_text(self.state.get_class(class_id).unwrap().classroom_type.to_string())
          //     .show_ui(ui, |ui| {
          //       for classroom_type_variant in enum_iterator::all::<ClassroomType>() {
          //         ui.selectable_value(
          //           &mut class.classroom_type,
          //           classroom_type_variant,
          //           classroom_type_variant.to_string(),
          //         );
          //       }
          //     });
          // });

          // TODO: Fix
          // ui.horizontal(|ui| {
          //   ui.label("Semestre");
          //   ComboBox::from_id_source(format!("semester_selector_{}", class_id))
          //     .selected_text(class.semester.to_string())
          //     .show_ui(ui, |ui| {
          //       for semester_variant in enum_iterator::all::<Semester>() {
          //         ui.selectable_value(
          //           &mut class.semester,
          //           semester_variant,
          //           semester_variant.to_string(),
          //         );
          //       }
          //     });
          // });

          // TODO: Fix
          // ui.horizontal(|ui| {
          //   ui.label("Groupo");
          //   ComboBox::from_id_source(format!("group_selector_{}", class_id))
          //     .selected_text(class.group.to_string())
          //     .show_ui(ui, |ui| {
          //       for group_variant in enum_iterator::all::<Group>() {
          //         ui.selectable_value(&mut class.group, group_variant, group_variant.to_string());
          //       }
          //     });
          // });

          ui.horizontal(|ui| {
            ui.label("Profesor");
            ui.label(format!(
              "{}",
              self.state.get_class(class_id).unwrap().get_professor_id()
            ));
            ComboBox::from_id_source(format!("professor_selector_{}", class_id.clone()))
              .selected_text(
                self
                  .state
                  .get_professor_metadata(
                    *self.state.get_class(class_id).unwrap().get_professor_id(),
                  )
                  .map(|professor_metadata| professor_metadata.name.as_str())
                  .unwrap_or("Undefined Professor"),
              )
              .show_ui(ui, |ui| {
                let num_professors = self.state.get_num_professors();
                let selected_professor_id =
                  *self.state.get_class(class_id).unwrap().get_professor_id();
                for professor_id in 0..num_professors {
                  if ui
                    .selectable_label(
                      professor_id == selected_professor_id,
                      self
                        .state
                        .get_professor_metadata(professor_id)
                        .unwrap()
                        .name
                        .as_str(),
                    )
                    .changed()
                  {
                    // TODO Set selected professor.
                  }
                }
              })
          });
          ui.horizontal(|ui| {
            let original_class_hours = *self.state.get_class(class_id).unwrap().get_class_hours();
            let mut class_hours = original_class_hours;
            ui.add(
              egui::Slider::new(&mut class_hours, 0..=20).text(to_human_time(original_class_hours)),
            );
            self
              .state
              .get_class_entry_mut(class_id)
              .unwrap()
              .set_hours(class_hours);
          });
          ui.separator();
        }
      });
    if ui.button("+").clicked() {
      self.state.add_new_class();
    }
  }
}

fn to_human_time(class_hours: u8) -> String {
  // each unit is worth 30 minutes
  let hours = class_hours / 2;
  let half_hours = class_hours % 2;
  match (hours > 0, half_hours > 0) {
    (true, true) => format!("{} hr. 30 min.", hours),
    (true, false) => format!("{} hr.", hours),
    (false, true) => "30 min.".to_string(),
    (false, false) => "0 min.".to_string(),
  }
}
