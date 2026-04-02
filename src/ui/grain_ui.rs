use crate::granular::grain::EnvelopeMode;
use crate::granular::GranularParams;
use crate::ui::{call_on_change, send_params};
use egui::{ComboBox, Slider, Ui, Widget};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct GranularUi {
    params: GranularParams,
    gate: bool,
    buf_len: usize, // The length of the buffer these params operate on
    sender: Sender<GranularParams>,
    gate_sender: Sender<bool>,
}

impl GranularUi {
    pub fn new(sender: Sender<GranularParams>, gate_sender: Sender<bool>, buf_len: usize) -> Self {
        Self {
            params: Default::default(),
            gate: true,
            buf_len,
            sender,
            gate_sender,
        }
    }

    fn update_params(&self) {
        send_params(&self.sender, self.params.clone())
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Grain Controls");
            ui.horizontal(|ui| {
                let start = Slider::new(&mut self.params.start, 0..=(self.buf_len - 1))
                    .drag_value_speed(50.0)
                    .text("Start")
                    .ui(ui);

                // Min length of 25ms at 45khz, max of 8 seconds, or the length of the buffer
                let length = Slider::new(
                    &mut self.params.grain_length,
                    1100..=(44000 * 8).min(self.buf_len - 1),
                )
                .drag_value_speed(10.0)
                .text("Length")
                .ui(ui);

                if ui.button("Scan").clicked() {
                    let state = self.params.scan.unwrap_or(false);
                    if state {
                        println!("Scan off!");
                    } else {
                        println!("Scan on!");
                    }
                    self.params.scan = Some(!state);
                    send_params(
                        &self.sender,
                        GranularParams {
                            scan: Some(!state),
                            ..self.params.clone()
                        },
                    );
                }

                call_on_change(|| self.update_params(), &[start, length]);
            });

            ui.heading("Envelope Controls");
            ui.horizontal(|ui| {
                let spread = Slider::new(&mut self.params.grain_spread, 500..=self.buf_len)
                    .drag_value_speed(1.0)
                    .text("Spread")
                    .ui(ui);

                let gain = Slider::new(&mut self.params.gain, 0.0..=2.0)
                    .drag_value_speed(0.01)
                    .text("Gain")
                    .ui(ui);

                let density = Slider::new(&mut self.params.density, 0.10..=48.00)
                    .drag_value_speed(0.01)
                    .text("Density")
                    .ui(ui);

                call_on_change(|| self.update_params(), &[density, spread, gain])
            });
            let msg = if self.gate { "Pause" } else { "Play" };
            ui.horizontal(|ui| {
                if ui.button(msg).clicked() {
                    self.gate = !self.gate;
                    self.gate_sender
                        .send(self.gate)
                        .expect("Failed to send gate");
                }

                ComboBox::from_label("Envelope type")
                    .selected_text(format!("{:?}", self.params.envelope_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.params.envelope_mode,
                            EnvelopeMode::Smooth,
                            "Smooth",
                        );
                        ui.selectable_value(
                            &mut self.params.envelope_mode,
                            EnvelopeMode::Exp,
                            "Exp",
                        );
                    });

                let mut response_list = vec![];

                match &self.params.envelope_mode {
                    EnvelopeMode::Smooth => {}
                    EnvelopeMode::Exp => {
                        let sharpness_slider =
                            Slider::new(&mut self.params.envelope_sharpness, 0.0..=1.00)
                                .drag_value_speed(0.01)
                                .text("Sharpness")
                                .ui(ui);

                        let shape_slider =
                            Slider::new(&mut self.params.envelope_shape, 0.01..=0.99)
                                .drag_value_speed(0.01)
                                .text("Shape")
                                .ui(ui);

                        // When the sliders are present, push the results so changes are listened for
                        response_list.push(sharpness_slider);
                        response_list.push(shape_slider);
                    }
                }

                call_on_change(|| self.update_params(), &response_list)
            })
        });
    }
}
