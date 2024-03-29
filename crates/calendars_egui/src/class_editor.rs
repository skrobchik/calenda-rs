use egui::{ComboBox, ScrollArea, TextEdit};
use serde::{Deserialize, Serialize};

use calendars_core::{ClassId, ClassroomType, Group, SchoolSchedule, Semester};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClassEditor {
  pub search_text: String,
  pub open: bool,
}

impl Default for ClassEditor {
  fn default() -> Self {
    Self {
      search_text: Default::default(),
      open: true,
    }
  }
}

impl ClassEditor {
  pub fn show(&mut self, ctx: &egui::Context, state: &mut SchoolSchedule) {
    let mut open = self.open;
    egui::Window::new("Clases")
      .open(&mut open)
      .resizable(true)
      .show(ctx, |ui| {
        self.ui(ui, state);
      });
    self.open = open;
  }

  fn class_entry(&mut self, ui: &mut egui::Ui, state: &mut SchoolSchedule, class_id: ClassId) {
    let class_name = &state.get_class_metadata(class_id).unwrap().name;
    if !class_name
      .to_lowercase()
      .contains(&self.search_text.to_lowercase())
    {
      return;
    }

    ui.horizontal(|ui| {
      ui.color_edit_button_srgba(&mut state.get_class_metadata_mut(class_id).unwrap().color);
      TextEdit::singleline(&mut state.get_class_metadata_mut(class_id).unwrap().class_code)
        .char_limit(4)
        .desired_width(50.0)
        .show(ui);
      ui.text_edit_singleline(&mut state.get_class_metadata_mut(class_id).unwrap().name);
    });

    {
      let mut optativa = state.get_class(class_id).unwrap().is_optative();
      ui.checkbox(&mut optativa, "Optativa");
      state
        .get_class_entry(class_id)
        .unwrap()
        .set_optative(optativa);
    }

    ui.horizontal(|ui| {
      ui.label("Aula");
      ComboBox::from_id_source(format!("classroom_type_selector_{}", usize::from(class_id)))
        .selected_text(
          state
            .get_class(class_id)
            .unwrap()
            .get_classroom_type()
            .to_string(),
        )
        .show_ui(ui, |ui| {
          let mut classroom_type = *state.get_class(class_id).unwrap().get_classroom_type();
          for classroom_type_variant in enum_iterator::all::<ClassroomType>() {
            ui.selectable_value(
              &mut classroom_type,
              classroom_type_variant,
              classroom_type_variant.to_string(),
            );
          }
          state
            .get_class_entry(class_id)
            .unwrap()
            .set_classroom_type(classroom_type);
        });
    });

    ui.horizontal(|ui| {
      ui.label("Semestre");
      ComboBox::from_id_source(format!("semester_selector_{}", usize::from(class_id)))
        .selected_text(
          state
            .get_class(class_id)
            .unwrap()
            .get_semester()
            .to_string(),
        )
        .show_ui(ui, |ui| {
          let mut semester = *state.get_class(class_id).unwrap().get_semester();
          for semester_variant in enum_iterator::all::<Semester>() {
            ui.selectable_value(
              &mut semester,
              semester_variant,
              semester_variant.to_string(),
            );
          }
          state
            .get_class_entry(class_id)
            .unwrap()
            .set_semester(semester);
        });
    });

    ui.horizontal(|ui| {
      ui.label("Groupo");
      ComboBox::from_id_source(format!("group_selector_{}", usize::from(class_id)))
        .selected_text(state.get_class(class_id).unwrap().get_group().to_string())
        .show_ui(ui, |ui| {
          let mut group = *state.get_class(class_id).unwrap().get_group();
          for group_variant in enum_iterator::all::<Group>() {
            ui.selectable_value(&mut group, group_variant, group_variant.to_string());
          }
          state.get_class_entry(class_id).unwrap().set_group(group);
        });
    });

    ui.horizontal(|ui| {
      ui.label("Profesor");
      ui.label(format!(
        "{}",
        state.get_class(class_id).unwrap().get_professor_id()
      ));
      ComboBox::from_id_source(format!("professor_selector_{}", usize::from(class_id)))
        .selected_text(
          state
            .get_professor_metadata(*state.get_class(class_id).unwrap().get_professor_id())
            .map(|professor_metadata| professor_metadata.name.as_str())
            .unwrap_or("Undefined Professor"),
        )
        .show_ui(ui, |ui| {
          let num_professors = state.get_num_professors();
          let selected_professor_id = *state.get_class(class_id).unwrap().get_professor_id();
          for professor_id in 0..num_professors {
            if ui
              .selectable_label(
                professor_id == selected_professor_id,
                state
                  .get_professor_metadata(professor_id)
                  .unwrap()
                  .name
                  .as_str(),
              )
              .clicked()
            {
              state
                .get_class_entry(class_id)
                .unwrap()
                .set_professor_id(professor_id);
            }
          }
        })
    });
    ui.horizontal(|ui| {
      let original_class_hours = *state.get_class(class_id).unwrap().get_class_hours();
      let mut class_hours = original_class_hours;
      ui.add(egui::Slider::new(&mut class_hours, 0..=20).text(to_human_time(original_class_hours)));
      state
        .get_class_entry(class_id)
        .unwrap()
        .set_hours(class_hours);
    });
    ui.separator();
  }

  fn ui(&mut self, ui: &mut egui::Ui, state: &mut SchoolSchedule) {
    ui.text_edit_singleline(&mut self.search_text);
    ui.separator();
    let num_classes = state.get_num_classes();
    ScrollArea::vertical()
      .auto_shrink([false; 2])
      .max_height(500.0)
      .show(ui, |ui| {
        for class_id in 0..num_classes {
          self.class_entry(ui, state, class_id.try_into().unwrap());
        }
      });
    if ui.button("+").clicked() {
      state.add_new_class();
    }
  }
}

fn to_human_time(class_hours: u8) -> String {
  // each unit is worth 60 minutes
  let hours = class_hours;
  match hours == 1 {
    true => "1 hr.".to_string(),
    false => format!("{} hrs.", hours),
  }
}
