use crate::delay::{DelayParams, FeedbackParams};
use crate::saturation::SaturationMode;
use crate::ui::{call_on_change, send_params};
use egui::{ComboBox, Slider, Ui, Widget};
use std::sync::mpsc::Sender;

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

    fn update_params(&self) {
        send_params(&self.sender, self.params.clone())
    }

    fn update_fb_params(&self) {
        send_params(&self.fb_sender, self.fb_params.clone())
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
                self.update_params();
            }

            if ui.button("Saturate").clicked() {
                self.fb_params.saturate = !self.fb_params.saturate;
                self.update_fb_params();
            }

            if ui.button("Pitch taps").clicked() {
                self.params.pitch = !self.params.pitch;
                self.update_params();
            }

            call_on_change(|| self.update_params(), &[mix, feedback, time_l, time_r]);

            if drive.changed() {
                self.update_fb_params();
            }
        });
    }
}
