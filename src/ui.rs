use crate::delay::{DelayParams, FeedbackParams};
use crate::grain::GranularParams;
use crate::lsystem::{LSystem, Turtle};
use crate::saturation::SaturationMode;
use eframe::emath;
use eframe::emath::{pos2, Pos2, Rect, Vec2};
use eframe::epaint::{Color32, Stroke};
use egui::{Response, Sense, Shape, Ui, Widget};
use rand::{random, Rng};
use rand_core::SeedableRng;
use rand_pcg::Pcg64Mcg;
use std::sync::mpsc::Sender;

pub struct LSystemUi {
    pub color: Color32,
    canvas_size: f32,
    systems: Vec<LSystem>,
    pub system: usize,
    lines: Vec<Vec<(Pos2, f32)>>,
    angle_seed: u64,
    length_seed: u64,
    pub angle: f32,
    pub angle_rand: f32,
    pub length_rand: f32,
    pub len: f32,
    pub width_falloff: f32,
    pub base_width: f32,
    pub min_width: f32,
}

impl LSystemUi {
    pub fn new(systems: Vec<LSystem>) -> Self {
        Self {
            color: Color32::from_hex("#99a933").unwrap(),
            canvas_size: 500.0,
            lines: vec![],
            systems,
            system: 0,
            angle_seed: 123123123,
            length_seed: 123123123,
            angle: 25.0,
            angle_rand: 2.0,
            length_rand: 1.0,
            len: 2.0,
            width_falloff: 0.7,
            base_width: 14.0,
            min_width: 1.5,
        }
    }

    pub fn system(&self) -> &LSystem {
        &self.systems[self.system]
    }

    fn create_lines(
        &self,
        base_width: f32,
        min_width: f32,
        width_falloff: f32,
    ) -> Vec<Vec<(Pos2, f32)>> {
        let mut turtle = Turtle::new(base_width, min_width, width_falloff);
        let mut lines = vec![];
        let mut current_line: Vec<(Pos2, f32)> = vec![(pos2(0.0, 0.0), base_width)];

        let mut rng = Pcg64Mcg::seed_from_u64(self.angle_seed);

        for block in self.system().encoded() {
            if block.chars().all(|c| c.is_ascii_digit()) {
                let run_len = block.parse::<u32>().expect("Failed to parse run as u32") as f32;
                let rand = rng.random::<f32>() * self.length_rand * self.len;
                turtle.forward((self.len + rand) * run_len);
            } else {
                for c in block.chars() {
                    if c == ']' {
                        turtle.pop();
                        lines.push(current_line.clone());
                        current_line = vec![turtle.get()]
                    } else {
                        match c {
                            'x' => {}
                            'f' => {
                                let rand = rng.random::<f32>() * self.length_rand * self.len;
                                turtle.forward(self.len + rand);
                            }
                            '+' => {
                                let rand =
                                    rng.random::<f32>() * 2.0 * self.angle_rand - self.angle_rand;
                                turtle.rotate(self.angle + rand);
                            }
                            '-' => {
                                let rand =
                                    rng.random::<f32>() * 2.0 * self.angle_rand - self.angle_rand;
                                turtle.rotate(-self.angle + rand);
                            }
                            '[' => {
                                turtle.push();
                            }
                            s => panic!("Invalid symbol: {s} found in L-System!"),
                        };
                        current_line.push(turtle.get())
                    }
                }
                lines.push(current_line.clone());
            }
        }

        lines
    }

    pub fn randomise_seed(&mut self) {
        self.angle_seed = random::<u64>();
        self.length_seed = random::<u64>();
    }

    fn create_branch(line: &[(Pos2, f32)], colour: Color32) -> Vec<Shape> {
        let mut shapes = vec![];
        let branch_len = line.len() - 1;
        for i in 0..branch_len {
            let (first, width) = line[i];
            let (second, _) = line[i + 1];
            shapes.push(Shape::line(vec![first, second], Stroke::new(width, colour)))
        }

        shapes
    }

    pub fn plant_window(&mut self, ui: &mut Ui) -> Response {
        // Allocate space for our widget
        let (response, painter) =
            ui.allocate_painter(Vec2::splat(self.canvas_size), Sense::hover());

        painter.rect_filled(response.rect, 5.0, Color32::WHITE);

        let transform = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        let map_coord = |p: Pos2| pos2(p.x + self.canvas_size / 2.0, self.canvas_size - p.y);

        // Encoded lines for 6 iteration system is a 17% reduction
        self.lines = self.create_lines(self.base_width, self.min_width, self.width_falloff);

        for (i, line) in self.lines.iter().enumerate() {
            let point_widths = line
                .iter()
                .map(|(point, width)| (transform * map_coord(*point), *width))
                .collect::<Vec<_>>();

            let scalar = (self.lines.len() - i) as f32 * 64.0 / self.lines.len() as f32;
            let branch = Self::create_branch(
                &point_widths,
                Color32::blend(
                    self.color,
                    Color32::from_rgba_premultiplied(0, 0, 0, scalar as u8),
                ),
            );
            painter.extend(branch);
        }

        response
    }

    pub fn plant_ui(&mut self, ui: &mut Ui) {
        egui::Slider::new(&mut self.angle, 0.0..=65.0)
            .text("Angle")
            .ui(ui);
        egui::Slider::new(&mut self.len, 0.1..=6.0)
            .text("Length")
            .ui(ui);
        egui::Slider::new(&mut self.angle_rand, 0.0..=65.0)
            .text("Angle randomise")
            .ui(ui);
        egui::Slider::new(&mut self.length_rand, 0.0..=2.0)
            .text("Length randomise")
            .ui(ui);
        egui::Slider::new(&mut self.system, 0..=3)
            .text("System")
            .ui(ui);
        egui::Slider::new(&mut self.width_falloff, 0.0..=2.0)
            .drag_value_speed(0.001)
            .text("Width Falloff")
            .ui(ui);
        egui::Slider::new(&mut self.base_width, 1.0..=30.0)
            .drag_value_speed(0.001)
            .text("Base Width")
            .ui(ui);
        egui::Slider::new(&mut self.min_width, 0.5..=30.0)
            .drag_value_speed(0.001)
            .text("Min Width")
            .ui(ui);
        if ui.button("Randomise").clicked() {
            self.randomise_seed();
        };
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

            let drive = egui::Slider::new(&mut self.fb_params.drive, 0.01..=2.00)
                .text("Drive")
                .ui(ui);

            egui::ComboBox::from_label("Saturation Circuit")
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
