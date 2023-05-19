use egui::{ComboBox, ScrollArea};

use crate::school_schedule::{ClassroomType, SchoolSchedule, Semester, Group};

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
      .resizable(true)
      .show(ctx, |ui| {
        self.ui(ui);
      });
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) {
    ui.separator();
    let (mut classes, professors) = self.state.get_classes_and_professors_mut();
    let text_style = egui::TextStyle::Body;
    let num_rows = classes.len();
    let row_height = 6.0 * ui.spacing().interact_size.y + ui.text_style_height(&text_style);
    ScrollArea::vertical()
      .auto_shrink([false; 2])
      .max_height(500.0)
      .show_rows(ui, row_height, num_rows, |ui, row_range| {
        let class_range = classes.get_mut(row_range).unwrap();
        for (class, metadata, class_id) in class_range.iter_mut() {
          ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut metadata.color);
            ui.text_edit_singleline(&mut metadata.name);
          });
          ui.horizontal(|ui| {
            ui.label("Aula");
            ComboBox::from_id_source(format!("classroom_type_selector_{}", class_id))
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
            ui.label("Semestre");
            ComboBox::from_id_source(format!("semester_selector_{}", class_id))
              .selected_text(class.semester.to_string())
              .show_ui(ui, |ui| {
                for semester_variant in enum_iterator::all::<Semester>() {
                  ui.selectable_value(
                    &mut class.semester,
                    semester_variant,
                    semester_variant.to_string(),
                  );
                }
              });
          });
          ui.horizontal(|ui| {
            ui.label("Groupo");
            ComboBox::from_id_source(format!("group_selector_{}", class_id))
              .selected_text(class.group.to_string())
              .show_ui(ui, |ui| {
                for group_variant in enum_iterator::all::<Group>() {
                  ui.selectable_value(
                    &mut class.group,
                    group_variant,
                    group_variant.to_string(),
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
            let curr_class_hours = class.class_hours;
            ui.add(egui::Slider::new(&mut class.class_hours, 0..=20).text(to_human_time(curr_class_hours)));
          });
          ui.separator();
        }
      });
    if ui.button("+").clicked() {
      self.state.add_new_class();
    }
    self.state.fill_classes();
  }
}

fn to_human_time(class_hours: u8) -> String{
  // each unit is worth 30 minutes
  let hours = class_hours / 2;
  let half_hours = class_hours % 2;
  match (hours > 0, half_hours > 0) {
    (true, true) => format!("{} hr. 30 min.", hours),
    (true, false) => format!("{} hr.", hours),
    (false, true) => format!("30 min."),
    (false, false) => format!("0 min."),
  }
}