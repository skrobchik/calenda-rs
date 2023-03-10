use egui::ScrollArea;

use crate::school_schedule::SchoolSchedule;

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
    let classes = self.state.get_classes_mut();
    ui.label("Classes");
    let text_style = egui::TextStyle::Body;
    ScrollArea::new([false, true]).show_rows(
      ui,
      ui.text_style_height(&text_style),
      classes.len(),
      |ui, row_range| {
        let class_range = classes.get(row_range).unwrap();
        for (class, metadata) in class_range {
          ui.label(format!("Clase \"{}\"", metadata.name));
        }
      },
    );
    if ui.button("+").clicked() {
      self.state.add_new_class();
    }
  }
}
