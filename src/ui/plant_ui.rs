use crate::lsystem::{LSystem, Turtle};
use crate::plant::Plant;
use eframe::emath::{pos2, Pos2, Rect, RectTransform, Vec2};
use eframe::epaint::{Color32, Shape, Stroke};
use egui::{Response, Sense, Slider, Ui, Widget};
use rand::prelude::IndexedRandom;
use rand::{random, Rng};
use rand_core::SeedableRng;
use rand_pcg::{Mcg128Xsl64, Pcg64Mcg};
use std::f32::consts::PI;
use std::sync::mpsc::Sender;

pub struct LSystemUi {
    canvas_size: f32,
    plants: Vec<Plant>,
    pub current_plant: usize,
    plant_data: PlantData,
    angle_seed: u64,
    length_seed: u64,
    leaf_seed: u64,
    pub angle: f32,
    pub angle_rand: f32,
    pub length_rand: f32,
    pub len: f32,
    pub width_falloff: f32,
    pub base_width: f32,
    pub min_width: f32,
    pub leaf_length: f32,
    pub leaf_bias: f32,
    pub leaf_width: f32,
    pub leaf_rand: f32,
    sender: Sender<Vec<Pos2>>,
}

#[derive(Default)]
struct PlantData {
    pub shapes: Vec<Vec<Shape>>,
    pub branch_points: Vec<Pos2>,
}

impl LSystemUi {
    pub fn new(sender: Sender<Vec<Pos2>>) -> Self {
        Self {
            canvas_size: 500.0,
            plants: vec![Plant::tree_1(), Plant::tree_2(), Plant::tree_3(), Plant::tree_4()],
            current_plant: 0,
            plant_data: Default::default(),
            angle_seed: 123123123,
            length_seed: 123123123,
            leaf_seed: 123123123,
            angle: 25.0,
            angle_rand: 2.0,
            length_rand: 1.0,
            len: 2.0,
            width_falloff: 0.7,
            base_width: 14.0,
            min_width: 1.5,
            leaf_length: 20.0,
            leaf_bias: -0.3,
            leaf_width: 6.0,
            leaf_rand: 0.0,
            sender,
        }
    }

    // Attempt at automatically scaling up lower iterations so they are more similar in height
    // TODO Model this with the system growth function so that it's more accurate
    fn scaled_length(&self) -> f32 {
        self.len * 350.0 / (4.0 * self.plant().system.current_iteration as f32).powi(2)
    }

    pub fn plant(&self) -> &Plant {
        &self.plants[self.current_plant]
    }

    pub fn plant_mut(&mut self) -> &mut Plant {
        &mut self.plants[self.current_plant]
    }

    // Create the shapes from an L-System
    fn create_plant_data(
        &self,
        base_width: f32,
        min_width: f32,
        width_falloff: f32,
        transform: RectTransform,
    ) -> PlantData {
        let mut turtle = Turtle::new(base_width, min_width, width_falloff);
        let mut shapes = vec![];
        let mut current_line: Vec<(Pos2, f32)> = vec![(pos2(0.0, 0.0), base_width)];
        let mut branch_points = vec![];

        let mut rng = Pcg64Mcg::seed_from_u64(self.angle_seed);
        let plant = self.plant();

        for block in plant.system.encoded() {
            if block.chars().all(|c| c.is_ascii_digit()) {
                let run_len = block.parse::<u32>().expect("Failed to parse run as u32") as f32;
                let rand = rng.random::<f32>() * self.length_rand * self.scaled_length();
                turtle.forward((self.scaled_length() + rand) * run_len);
            } else {
                for c in block.chars() {
                    if c == ']' {
                        turtle.pop();
                        let point_widths = current_line
                            .iter()
                            .map(|(point, width)| (transform * self.map_coord(*point), *width))
                            .collect::<Vec<_>>();

                        let branch = Self::create_branch(&point_widths, plant.branch_colour);
                        shapes.push(branch);
                        current_line = vec![turtle.get()]
                    } else {
                        match c {
                            'l' => {
                                let pos = transform * self.map_coord(turtle.get().0);
                                let leaf = self.leaf(
                                    pos,
                                    self.leaf_length,
                                    self.leaf_bias,
                                    self.leaf_width,
                                    -turtle.angle(),
                                    &mut rng,
                                );
                                shapes.push(vec![leaf]);
                            }
                            'x' => {}
                            'f' => {
                                let rand =
                                    rng.random::<f32>() * self.length_rand * self.scaled_length();
                                turtle.forward(self.scaled_length() + rand);
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
                                branch_points.push(self.map_coord(turtle.get().0));
                            }
                            s => panic!("Invalid symbol: {s} found in L-System!"),
                        };
                        current_line.push(turtle.get())
                    }
                }
                let point_widths = current_line
                    .iter()
                    .map(|(point, width)| (transform * self.map_coord(*point), *width))
                    .collect::<Vec<_>>();

                let branch = Self::create_branch(&point_widths, plant.branch_colour);
                shapes.push(branch);
            }
        }

        PlantData {
            shapes,
            branch_points,
        }
    }

    pub fn randomise(&mut self) {
        self.angle_seed = random::<u64>();
        self.length_seed = random::<u64>();
        self.leaf_seed = random::<u64>();
        self.plant_mut().system.recompute();
    }

    fn create_trapezium(
        start: Pos2,
        end: Pos2,
        start_width: f32,
        end_width: f32,
        colour: Color32,
    ) -> Shape {
        let perp = (end - start).rot90().normalized();
        let points = vec![
            pos2(
                start.x - perp.x * start_width / 2.0,
                start.y - perp.y * start_width / 2.0,
            ),
            pos2(
                end.x - perp.x * end_width / 2.0,
                end.y - perp.y * end_width / 2.0,
            ),
            pos2(
                end.x + perp.x * end_width / 2.0,
                end.y + perp.y * end_width / 2.0,
            ),
            pos2(
                start.x + perp.x * start_width / 2.0,
                start.y + perp.y * start_width / 2.0,
            ),
        ];
        Shape::convex_polygon(points, colour, Stroke::NONE)
    }

    fn create_branch(line: &[(Pos2, f32)], colour: Color32) -> Vec<Shape> {
        let mut shapes = vec![];
        let branch_len = line.len() - 1;
        for i in 0..(branch_len) {
            let (first, width1) = line[i];
            let (second, width2) = line[i + 1];

            shapes.push(Self::create_trapezium(
                first, second, width1, width2, colour,
            ));
        }

        shapes
    }

    fn map_coord(&self, p: Pos2) -> Pos2 {
        pos2(p.x + self.canvas_size / 2.0, self.canvas_size - p.y)
    }

    fn leaf(
        &self,
        pos: Pos2,
        len: f32,
        offset: f32,
        width: f32,
        angle: f32,
        rng: &mut Mcg128Xsl64,
    ) -> Shape {
        const NUM_POINTS: usize = 12;
        let len_rand = self.leaf_rand * len * 0.7 * (rng.random::<f32>() - 0.5);
        let width_rand = self.leaf_rand * width * 0.5 * (rng.random::<f32>() - 0.5);
        let offset_rand = self.leaf_rand * 2.0 * (rng.random::<f32>() - 0.5);
        let colour_rand = self.leaf_rand * 65.0 * (rng.random::<f32>() - 0.2);

        let offset = offset + offset_rand;
        let len = len + len_rand;
        let width = width + width_rand;

        let offset = if offset == 0.0 {
            0.0001 // Exp of 0 gives division by 0 error
        } else {
            offset
        };

        // Functions composed for leaf shape
        let g = |x: f32| ((offset * x).exp() - 1.0) / (offset.exp() - 1.0);
        let f = |x: f32| width * (PI * g(x / len)).sin();

        let mut shape = vec![];
        let mut side = vec![];

        for i in 0..=NUM_POINTS {
            let x = i as f32 * len / NUM_POINTS as f32;
            let y = f(x);

            side.push(pos2(x, y));
            shape.push(pos2(x, y));
        }

        // Create mirror side of leaf
        let mut iter = side.iter().rev();
        iter.next(); // Removing last element, or it will be duplicated
        for point in iter {
            shape.push(pos2(point.x, -point.y))
        }

        // Rotation by standard rotation matrix
        let rotate_point = |p: Pos2| {
            pos2(
                (p.x * angle.cos()) - (p.y * angle.sin()),
                (p.x * angle.sin()) + (p.y * angle.cos()),
            )
        };

        let rotated = shape
            .iter()
            .map(|p| rotate_point(*p))
            .collect::<Vec<Pos2>>();

        let base_colour = self
            .plant()
            .leaf_colours
            .choose(rng)
            .expect("Colour Slice is empty!");

        let colour = base_colour.gamma_multiply_u8(120 + colour_rand as u8);
        let mut leaf = Shape::convex_polygon(rotated, colour, Stroke::NONE);
        leaf.translate(Vec2::new(pos.x, pos.y));
        leaf
    }

    pub fn plant_window(&mut self, ui: &mut Ui) -> Response {
        // Allocate space for plant window
        let (response, painter) =
            ui.allocate_painter(Vec2::splat(self.canvas_size), Sense::hover());

        painter.rect_filled(response.rect, 5.0, Color32::WHITE);

        let transform = RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        self.plant_data = self.create_plant_data(
            self.base_width,
            self.min_width,
            self.width_falloff,
            transform,
        );

        for item in &self.plant_data.shapes {
            painter.extend(item.clone());
        }

        self.sender
            .send(self.plant_data.branch_points.clone())
            .expect("Failed to send points to sequencer");
        response
    }

    pub fn plant_ui(&mut self, ui: &mut Ui) {
        ui.heading("Plant Controls");
        Slider::new(&mut self.angle, 0.0..=65.0)
            .text("Angle")
            .ui(ui);
        Slider::new(&mut self.len, 0.1..=6.0).text("Length").ui(ui);
        Slider::new(&mut self.angle_rand, 0.0..=65.0)
            .text("Angle randomise")
            .ui(ui);
        Slider::new(&mut self.length_rand, 0.0..=2.0)
            .text("Length randomise")
            .ui(ui);
        Slider::new(&mut self.current_plant, 0..=self.plants.len() - 1)
            .text("System")
            .ui(ui);
        let system = &mut self.plant_mut().system;
        Slider::new(&mut system.current_iteration, 0..=system.iterations)
            .text("Iterations")
            .ui(ui);
        Slider::new(&mut self.width_falloff, 0.0..=2.0)
            .drag_value_speed(0.001)
            .text("Width Falloff")
            .ui(ui);
        Slider::new(&mut self.base_width, 1.0..=30.0)
            .drag_value_speed(0.001)
            .text("Base Width")
            .ui(ui);
        Slider::new(&mut self.min_width, 0.5..=30.0)
            .drag_value_speed(0.001)
            .text("Min Width")
            .ui(ui);
        Slider::new(&mut self.leaf_length, 2.0..=70.0)
            .drag_value_speed(0.01)
            .text("Leaf Length")
            .ui(ui);
        Slider::new(&mut self.leaf_bias, -3.0..=3.0)
            .drag_value_speed(0.01)
            .text("Leaf Shape")
            .ui(ui);
        Slider::new(&mut self.leaf_width, 2.0..=70.0)
            .drag_value_speed(0.01)
            .text("Leaf Width")
            .ui(ui);
        Slider::new(&mut self.leaf_rand, 0.0..=1.0)
            .drag_value_speed(0.001)
            .text("Leaf Rand")
            .ui(ui);
        if ui.button("Randomise").clicked() {
            self.randomise();
        };
    }
}
