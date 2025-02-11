use crate::delay::DelayParams;
use crate::grain::GranularParams;
use crate::lsystem::{LSystem, Turtle};
use eframe::emath;
use eframe::emath::{pos2, Pos2, Rect, Vec2};
use eframe::epaint::{Color32, Stroke};
use egui::{Response, Sense, Ui, Widget};
use rand::{random, Rng};
use rand_core::SeedableRng;
use rand_pcg::Pcg64Mcg;
use std::sync::mpsc::Sender;

pub struct LSystemUi {
    pub color: Color32,
    canvas_size: f32,
    pub system: LSystem,
    lines: Vec<Vec<Pos2>>,
    seed: u64,
    pub angle: f32,
    pub rand_amount: f32,
}

impl LSystemUi {
    pub fn new(color: egui::Color32, canvas_size: f32, system: LSystem) -> Self {
        Self {
            color,
            canvas_size,
            lines: vec![],
            system,
            seed: 123123123,
            angle: 25.0,
            rand_amount: 2.0,
        }
    }

    // Create vector of lines from system, using turtle commands
    fn create_lines(&self) -> Vec<Vec<Pos2>> {
        let mut turtle = Turtle::new();
        let mut lines = vec![];
        let mut current_line: Vec<Pos2> = vec![pos2(0.0, 0.0)];

        let mut rng = Pcg64Mcg::seed_from_u64(self.seed);

        for c in self.system.result.chars() {
            if c == ']' {
                turtle.pop();
                lines.push(current_line.clone());
                current_line = vec![turtle.pos()]
            } else {
                match c {
                    'x' => {}
                    'f' => {
                        turtle.forward(2.0);
                    }
                    '+' => {
                        let rand = rng.random::<f32>() * 2.0 * self.rand_amount - self.rand_amount;
                        turtle.rotate(self.angle + rand);
                    }
                    '-' => {
                        let rand = rng.random::<f32>() * 2.0 * self.rand_amount - self.rand_amount;
                        turtle.rotate(-self.angle + rand);
                    }
                    '[' => {
                        turtle.push();
                    }
                    s => panic!("Invalid symbol: {s} found in L-System!"),
                };
                current_line.push(turtle.pos())
            }
        }
        lines.push(current_line.clone());

        lines
    }

    pub fn randomise_seed(&mut self) {
        self.seed = random::<u64>();
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        // Allocate space for our widget
        let (response, painter) =
            ui.allocate_painter(Vec2::splat(self.canvas_size), Sense::hover());

        painter.rect_filled(response.rect, 5.0, Color32::WHITE);

        let transform = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let map_coord = |p: Pos2| pos2(p.x + self.canvas_size / 2.0, self.canvas_size - p.y);

        self.lines = self.create_lines();

        for line in &self.lines {
            let points = line
                .iter()
                .map(|point| transform * map_coord(*point))
                .collect();

            let shape = egui::Shape::line(points, Stroke::new(2.3, self.color));
            painter.add(shape);
        }

        response
    }
}

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
                let start = egui::Slider::new(&mut self.params.start, 0..=(self.buf_len - 1))
                    .text("Start")
                    .ui(ui);
                let length = egui::Slider::new(&mut self.params.grain_length, 0..=88000)
                    .text("Length")
                    .ui(ui);

                let pan = egui::Slider::new(&mut self.params.pan_spread, 0.0..=1.0)
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
                let density = egui::Slider::new(&mut self.params.grain_density, 2000..=44000)
                    .text("Density")
                    .ui(ui);
                let spread = egui::Slider::new(&mut self.params.grain_spread, 0.0..=1.0)
                    .text("Spread")
                    .ui(ui);

                let gain = egui::Slider::new(&mut self.params.gain, 0.0..=2.0)
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

pub struct DelayUi {
    params: DelayParams,
    sender: Sender<DelayParams>,
}

impl DelayUi {
    pub fn new(sender: Sender<DelayParams>) -> Self {
        Self {
            params: Default::default(),
            sender,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            let mix = egui::Slider::new(&mut self.params.mix, 0.0..=1.0)
                .text("Mix")
                .ui(ui);
            let feedback = egui::Slider::new(&mut self.params.feedback, 0.0..=0.999)
                .text("Feedback")
                .ui(ui);
            let time_l = egui::Slider::new(&mut self.params.time_l, 0.001..=5.00)
                .text("Left time")
                .ui(ui);
            let time_r = egui::Slider::new(&mut self.params.time_r, 0.001..=5.00)
                .text("Right Time")
                .ui(ui);

            if ui.button("Bypass").clicked() {
                self.params.bypass = !self.params.bypass;
                self.sender
                    .send(self.params.clone())
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
        });
    }
}
