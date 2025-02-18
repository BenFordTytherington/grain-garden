use std::sync::mpsc::Sender;
use egui::{ComboBox, Slider, Ui, Widget};
use crate::delay::{DelayParams, FeedbackParams};
use crate::saturation::SaturationMode;

pub struct DelayUi {
    params: DelayParams,
    sender: Sender<DelayParams>,
    fb_params: FeedbackParams,
    fb_sender: Sender<FeedbackParams>,
}

impl DelayUi {
    pub fn new(sender: Sender<DelayParams>, fb_sender: Sender<FeedbackParams>) -> Self {
        Self {
            params: Default::default(),
            sender,
            fb_params: Default::default(),
            fb_sender,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Delay Controls");
        ui.vertical_centered(|ui| {
            let mix = Slider::new(&mut self.params.mix, 0.0..=1.0)
                .text("Mix")
                .ui(ui);
            let feedback = Slider::new(&mut self.params.feedback, 0.0..=0.999)
                .text("Feedback")
                .ui(ui);
            let time_l = Slider::new(&mut self.params.time_l, 0.001..=5.00)
                .text("Left time")
                .ui(ui);
            let time_r = Slider::new(&mut self.params.time_r, 0.001..=5.00)
                .text("Right Time")
                .ui(ui);

            let drive = Slider::new(&mut self.fb_params.drive, 0.01..=2.00)
                .text("Drive")
                .ui(ui);

            ComboBox::from_label("Saturation Circuit")
                .selected_text(format!("{:?}", self.fb_params.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.fb_params.mode, SaturationMode::Tape, "Tape");
                    ui.selectable_value(&mut self.fb_params.mode, SaturationMode::Tube, "Tube");
                    ui.selectable_value(
                        &mut self.fb_params.mode,
                        SaturationMode::Transistor,
                        "Transistor",
                    );
                });

            if ui.button("Bypass").clicked() {
                self.params.bypass = !self.params.bypass;
                self.sender
                    .send(self.params.clone())
                    .expect("Failed to send params")
            }

            if ui.button("Saturate").clicked() {
                self.fb_params.saturate = !self.fb_params.saturate;
                self.fb_sender
                    .send(self.fb_params.clone())
                    .expect("Failed to send params")
            }

            if ui.button("Pitch taps").clicked() {
                self.params.pitch = !self.params.pitch;
                self.sender
                    .send(self.params.clone())
                    .expect("Failed to send params")
            }

            if mix.changed() | feedback.changed() | time_l.changed() | time_r.changed() {
                self.sender
                    .send(self.params.clone())
                    .expect("Failed to send params");
            }

            if drive.changed() {
                self.fb_sender
                    .send(self.fb_params.clone())
                    .expect("Failed to send params");
            }
        });
    }
}