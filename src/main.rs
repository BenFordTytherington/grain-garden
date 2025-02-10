mod delay;
mod dsp;
mod grain;
mod lsystem;
mod ui;

use crate::dsp::interleave;
use crate::grain::GranularEngine;
use crate::lsystem::LSystem;
use crate::ui::{DelayUi, GranularUi, LSystemUi};
use egui::{Color32, Id, Widget};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use std::sync::mpsc::channel;

struct App {
    granular_ui: GranularUi,
    lsystem_ui: LSystemUi,
    delay_ui: DelayUi,
}

impl App {
    fn new(
        _cc: &eframe::CreationContext<'_>,
        granular_ui: GranularUi,
        lsystem_ui: LSystemUi,
        delay_ui: DelayUi,
    ) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            granular_ui,
            lsystem_ui,
            delay_ui,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Redraws all the Ui elements
        egui::TopBottomPanel::top(Id::new("grain_controls"))
            .resizable(true)
            .min_height(100.0)
            .max_height(200.0)
            .show(ctx, |ui| self.granular_ui.ui(ui));

        egui::SidePanel::left(Id::new("delay_controls"))
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Delay Controls");
                self.delay_ui.ui(ui);
            });

        egui::SidePanel::right(Id::new("plant_controls"))
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Plant Controls");
                egui::Slider::new(&mut self.lsystem_ui.system.angle, 1.0..=45.0)
                    .text("Angle")
                    .ui(ui);
            });

        // Draw plant last so it occupies remaining screenspace
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Plant");
                self.lsystem_ui.ui(ui);
            });
        });
    }
}

fn main() -> eframe::Result {
    let (param_send, param_receive) = channel();
    let (gate_send, gate_receive) = channel();
    let (delay_send, delay_receive) = channel();

    let mut granny = GranularEngine::new("handpan.wav", param_receive, gate_receive, delay_receive);
    granny.init();
    let sample_len = granny.buffer_size();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Start audio thread
    std::thread::spawn(move || {
        loop {
            // Exit current loop early if the sink has enough samples to play
            if sink.len() >= 2 {
                continue;
            }

            let output: Vec<f32> = interleave(granny.process(2048)); // Arbitrary buffer size

            // Play the output buffer
            sink.append(SamplesBuffer::new(2, 44000, output));
        }
    });

    let granular_ui = GranularUi::new(param_send, gate_send, sample_len);

    let delay_ui = DelayUi::new(delay_send);

    // Barnsley fern
    let mut system = LSystem::new('x', vec!["x->f+[[x]-x]-f[-fx]+x", "f->ff"], 35.0);

    system.iterate(6);

    let lsystem_ui = LSystemUi::new(Color32::from_hex("#99a933").unwrap(), 500.0, system);

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Granular Plants",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, granular_ui, lsystem_ui, delay_ui)))),
    )?;

    Ok(())
}
