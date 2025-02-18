use std::sync::mpsc::Sender;
use egui::{Slider, Ui, Widget};
use crate::grain::GranularParams;

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

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Grain Controls");
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                let start = Slider::new(&mut self.params.start, 0..=(self.buf_len - 1))
                    .text("Start")
                    .ui(ui);
                let length = Slider::new(&mut self.params.grain_length, 0..=88000)
                    .text("Length")
                    .ui(ui);

                let pan = Slider::new(&mut self.params.pan_spread, 0.0..=1.0)
                    .text("Pan Spread")
                    .ui(ui);

                if ui.button("Scan").clicked() {
                    let state = self.params.scan.unwrap_or(false);
                    if state {
                        println!("Scan off!");
                    } else {
                        println!("Scan on!");
                    }
                    self.params.scan = Some(!state);
                    self.sender
                        .send(GranularParams {
                            scan: Some(!state),
                            ..self.params.clone()
                        })
                        .expect("Failed to send params");
                }

                if start.changed() | length.changed() | pan.changed() {
                    self.sender
                        .send(self.params.clone())
                        .expect("Failed to send params");
                }
            });
            ui.horizontal(|ui| {
                let density = Slider::new(&mut self.params.grain_density, 2000..=44000)
                    .text("Density")
                    .ui(ui);
                let spread = Slider::new(&mut self.params.grain_spread, 0.0..=1.0)
                    .text("Spread")
                    .ui(ui);

                let gain = Slider::new(&mut self.params.gain, 0.0..=2.0)
                    .text("Gain")
                    .ui(ui);

                if ui.button("Gate").is_pointer_button_down_on() && !self.gate {
                    self.gate_sender.send(true).expect("Failed to send gate");
                    self.gate = true;
                    println!("Sending gate on");
                } else if self.gate {
                    self.gate_sender.send(false).expect("Failed to send gate");
                    self.gate = false;
                    println!("Sending gate off");
                }

                if density.changed() | spread.changed() | gain.changed() {
                    self.sender
                        .send(self.params.clone())
                        .expect("Failed to send params");
                }
            });
        });
    }
}