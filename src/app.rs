use eframe::{egui, epi};
use egui::{Pos2, Color32, Stroke};
use rand::prelude::ThreadRng;
use std::f32::consts::PI;

use crate::{calendar_widget::CalendarWidget, calendars::CalendarState, simulation::Simulation};

pub struct MyApp {
    name: String,
    age: u32,
    simulation: Simulation,
    rng: ThreadRng
}

impl MyApp {
    pub fn new(simulation: Simulation, rng: ThreadRng) -> MyApp {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            simulation,
            rng
        }
    }
}

impl MyApp {
    fn temperature(x: f32) -> f32 {
        (1.0 / (x + 1.0)) - 0.5 * ((2.0 * PI * x).cos()).powi(2)
    }
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "My egui App"
    }
   
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        self.simulation.step(MyApp::temperature(self.simulation.get_step_count() as f32 / 1000000 as f32), &mut self.rng);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
            ui.add(CalendarWidget::new(self.simulation.get_current_state()))
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}