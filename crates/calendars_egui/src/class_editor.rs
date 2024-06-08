use egui::{Color32, ComboBox, RichText, ScrollArea, TextEdit};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use calendars_core::{
  enumflags2::BitFlags, strum::IntoEnumIterator, ClassKey, ClassroomType, Group, SchoolSchedule,
  Semester,
};

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

  fn class_entry(&mut self, ui: &mut egui::Ui, state: &mut SchoolSchedule, class_key: ClassKey) {
    let class_name = &state.get_class_metadata(class_key).unwrap().name;
    if !class_name
      .to_lowercase()
      .contains(&self.search_text.to_lowercase())
    {
      return;
    }

    ui.horizontal(|ui| {
      let rgba = state.get_class_metadata_mut(class_key).unwrap().rgba;
      let mut color = Color32::from_rgba_premultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
      ui.color_edit_button_srgba(&mut color);
      state.get_class_metadata_mut(class_key).unwrap().rgba = color.to_array();
      TextEdit::singleline(&mut state.get_class_metadata_mut(class_key).unwrap().class_code)
        .char_limit(4)
        .desired_width(50.0)
        .show(ui);
      ui.text_edit_singleline(&mut state.get_class_metadata_mut(class_key).unwrap().name);
    });

    ui.horizontal(|ui| {
      ui.vertical(|ui| {
      egui::Frame::default()
        .inner_margin(4.0)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .rounding(ui.visuals().widgets.noninteractive.rounding)
        .show(ui, |ui| {
          ui.vertical(|ui| {
            ui.label(RichText::new("Aulas Permitidas").strong()).on_hover_text(
              "El programa intentara asignar una aula de las categorias seleccionadas a esta clase.",
            );
            let curr_classroom_types = state
              .get_class(class_key)
              .unwrap()
              .allowed_classroom_types;
            let classroom_types = ClassroomType::iter()
              .map(|v| (v, curr_classroom_types.contains(v)))
              .collect_vec();
            let classroom_types = classroom_types
              .into_iter()
              .map(|(v, mut curr)| {
                if !matches!(v, ClassroomType::NotAssigned) {
                  ui.checkbox(&mut curr, v.to_string());
                }
                (v, curr)
              })
              .filter(|(_v, curr)| *curr)
              .map(|(v, _curr)| v)
              .collect_vec();
            let new_classroom_types = classroom_types
              .into_iter()
              .fold(BitFlags::empty(), |a, b| a | b);
            state
              .get_class_entry(class_key)
              .unwrap()
              .set_allowed_classroom_types(new_classroom_types);
          });
        });
      });
      ui.vertical(|ui| {
        {
          let mut optativa = state.get_class(class_key).unwrap().optative;
          ui.checkbox(&mut optativa, "Optativa");
          state
            .get_class_entry(class_key)
            .unwrap()
            .set_optative(optativa);
        }
        ui.horizontal(|ui| {
          ui.label("Semestre");
          ComboBox::from_id_source(format!("semester_selector_{:?}", class_key))
            .selected_text(
              state
                .get_class(class_key)
                .unwrap()
                .semester
                .to_string(),
            )
            .show_ui(ui, |ui| {
              let mut semester = state.get_class(class_key).unwrap().semester;
              for semester_variant in Semester::iter() {
                ui.selectable_value(
                  &mut semester,
                  semester_variant,
                  semester_variant.to_string(),
                );
              }
              state
                .get_class_entry(class_key)
                .unwrap()
                .set_semester(semester);
            });
        });

        ui.horizontal(|ui| {
          ui.label("Groupo");
          ComboBox::from_id_source(format!("group_selector_{:?}", class_key))
            .selected_text(state.get_class(class_key).unwrap().group.to_string())
            .show_ui(ui, |ui| {
              let mut group = state.get_class(class_key).unwrap().group;
              for group_variant in Group::iter() {
                ui.selectable_value(&mut group, group_variant, group_variant.to_string());
              }
              state.get_class_entry(class_key).unwrap().set_group(group);
            });
        });

        ui.horizontal(|ui| {
          ui.label("Profesor");
          ComboBox::from_id_source(egui::Id::new(("professor_combo_box", class_key)))
            .selected_text(
              state
                .get_professor_metadata(state.get_class(class_key).unwrap().professor_key)
                .map(|professor_metadata| professor_metadata.name.as_str())
                .unwrap_or("Undefined Professor"),
            )
            .show_ui(ui, |ui| {
              let selected_professor_id = state.get_class(class_key).unwrap().professor_key;
              let professor_keys = state
                .get_simulation_constraints()
                .professors.iter().map(|(k, v)| k)
                .collect_vec();
              for professor_id in professor_keys {
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
                    .get_class_entry(class_key)
                    .unwrap()
                    .set_professor_id(professor_id);
                }
              }
            })
        });
        ui.horizontal(|ui| {
          let original_class_hours = state.get_class(class_key).unwrap().class_hours;
          let mut class_hours = original_class_hours;
          ui.add(egui::Slider::new(&mut class_hours, 0..=20).text(to_human_time(original_class_hours)));
          state
            .get_class_entry(class_key)
            .unwrap()
            .set_hours(class_hours);
        });
      });
    });

    ui.separator();
  }

  fn ui(&mut self, ui: &mut egui::Ui, state: &mut SchoolSchedule) {
    ui.text_edit_singleline(&mut self.search_text);
    ui.separator();
    let class_keys = state.get_class_calendar().iter_class_keys().collect_vec();
    ScrollArea::vertical()
      .auto_shrink([false; 2])
      .max_height(500.0)
      .show(ui, |ui| {
        for class_key in class_keys.into_iter() {
          self.class_entry(ui, state, class_key);
        }
      });
    if ui.button("+").clicked() {
      let professor_key = state.get_simulation_constraints().professors.keys().next();
      let professor_key = professor_key.unwrap_or_else(|| state.add_new_professor());
      state.add_new_class(professor_key);
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
