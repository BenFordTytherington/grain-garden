use crate::granular::GranularParams;
use crate::ui::{call_on_change, send_params};
use egui::{Slider, Ui, Widget};
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
        ui.heading("Grain Controls");
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                let start = Slider::new(&mut self.params.start, 0..=(self.buf_len - 1))
                    .drag_value_speed(50.0)
                    .text("Start")
                    .ui(ui);

                let length = Slider::new(&mut self.params.grain_length, 0..=88000)
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

                call_on_change(|| self.update_params(), &[start, length])
            });
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

                if ui.button("Gate").clicked() {
                    self.gate = !self.gate;
                    self.gate_sender
                        .send(self.gate)
                        .expect("Failed to send gate");
                    let msg = if self.gate { "on" } else { "off" };
                    println!("Sending gate {}", msg);
                }

                call_on_change(|| self.update_params(), &[density, spread, gain])
            });
        });
    }
}
