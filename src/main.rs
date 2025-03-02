mod delay;
mod dsp;
mod granular;
mod lsystem;
mod plant;
mod saturation;
mod ui;

use crate::delay::StereoDelay;
use crate::dsp::{interleave, StereoFrame};
use crate::granular::GranularEngine;
use crate::lsystem::LSystem;
use crate::ui::{DelayUi, GranularUi, LSystemUi};
use eframe::epaint::FontFamily;
use egui::{CentralPanel, Color32, Context, Id, RichText, SidePanel, TopBottomPanel, Visuals};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;

struct App {
    granular_ui: GranularUi,
    lsystem_ui: LSystemUi,
    delay_ui: DelayUi,
}

impl App {
    fn new(
        cc: &eframe::CreationContext<'_>,
        granular_ui: GranularUi,
        lsystem_ui: LSystemUi,
        delay_ui: DelayUi,
    ) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        cc.egui_ctx.set_visuals(Visuals {
            panel_fill: Color32::from_rgb(41, 34, 37),
            override_text_color: Some(Color32::from_rgb(143, 137, 123)),
            ..Default::default()
        });

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "verdant".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/fonts/Verdant.ttf"
            ))),
        );
        fonts
            .families
            .entry(FontFamily::Name("verdant".into()))
            .or_default()
            .insert(0, "verdant".to_owned()); // Not main font

        cc.egui_ctx.set_fonts(fonts);

        Self {
            granular_ui,
            lsystem_ui,
            delay_ui,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Redraws all the Ui elements
        TopBottomPanel::top(Id::new("grain_controls"))
            .resizable(true)
            .min_height(100.0)
            .max_height(200.0)
            .show(ctx, |ui| self.granular_ui.ui(ui));

        SidePanel::left(Id::new("delay_controls"))
            .resizable(true)
            .show(ctx, |ui| {
                self.delay_ui.ui(ui);
            });

        SidePanel::right(Id::new("plant_controls"))
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    self.lsystem_ui.plant_ui(ui);
                })
            });

        // Draw plant last so it occupies remaining screen space
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("Grain Garden")
                        .heading()
                        .size(60.0)
                        .family(FontFamily::Name("verdant".into())),
                );
                self.lsystem_ui.plant_window(ui);
            });
        });
    }
}

fn main() -> eframe::Result {
    // Setup Channels for Ui and audio interaction
    let (param_send, param_receive) = channel();
    let (gate_send, gate_receive) = channel();
    let (delay_send, delay_receive) = channel();
    let (fb_send, fb_receive) = channel();
    let (seq_send, seq_receive) = channel();

    // Init granular engine
    let mut granny = GranularEngine::new(
        PathBuf::from("assets/audio/handpan_trimmed.wav"),
        param_receive,
        gate_receive,
        seq_receive,
    );
    granny.init();
    let sample_len = granny.buffer_size();

    let mut delay = StereoDelay::new(2.45625, 1.53312, 44000, 0.2, 0.5, delay_receive, fb_receive);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Start audio thread
    // This could potentially be optimised somehow by implementing Source and Sample for my objects
    // The magic number `8` just seems to reduce pops and clicks by not starving the buffer
    std::thread::spawn(move || {
        let mut buffer: Vec<StereoFrame> = vec![StereoFrame::new(0.0); 512];
        loop {
            // Exit current loop early if the sink has enough samples to play
            if sink.len() >= 8 {
                continue;
            }

            granny.process_block(buffer.as_mut_slice());
            delay.process_block(buffer.as_mut_slice());
            let output: Vec<f32> = interleave(buffer.as_slice());

            // Play the output buffer
            sink.append(SamplesBuffer::new(2, 44000, output));
        }
    });

    // Create Ui widgets
    let granular_ui = GranularUi::new(param_send, gate_send, sample_len);
    let delay_ui = DelayUi::new(delay_send, fb_send);
    let lsystem_ui = LSystemUi::new(seq_send);

    // Run the eframe app
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Granular Plants",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, granular_ui, lsystem_ui, delay_ui)))),
    )?;

    Ok(())
}
