mod delay;
mod dsp;
mod grain;
mod lsystem;

// use crate::grain::{Granulizer, GranulizerParams};
use crate::lsystem::LSystem;
use eframe::emath;
use eframe::epaint::Stroke;
use egui::{pos2, Color32, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};
// use rand::random;
// use rodio::{OutputStream, Sink};
// use std::path::PathBuf;
// use std::sync::mpsc::{channel, Sender};

pub struct PointCanvas {
    lines: Vec<Vec<Pos2>>,
    color: Color32,
}

impl PointCanvas {
    pub fn new(lines: Vec<Vec<Pos2>>, color: egui::Color32) -> Self {
        Self { lines, color }
    }
}

impl Widget for PointCanvas {
    fn ui(self, ui: &mut Ui) -> Response {
        // Allocate space for our widget
        let (mut response, painter) = ui.allocate_painter(Vec2::splat(600.0), Sense::hover());

        painter.rect_filled(response.rect, 5.0, Color32::WHITE);

        let transform = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.size()),
            response.rect,
        );

        for line in self.lines {
            let points = line
                .iter()
                .map(|point| map_coord(transform * *point))
                .collect();

            let shape = egui::Shape::line(points, Stroke::new(1.0, self.color));
            painter.add(shape);
        }

        response
    }
}

pub fn map_coord(p: Pos2) -> Pos2 {
    // hardcode width as 600
    pos2(p.x + 300.0, 600.0 - p.y)
}

#[derive(Default)]
struct App {
    lines: Vec<Vec<Pos2>>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>, points: Vec<Vec<Pos2>>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self { lines: points }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");

            PointCanvas::new(self.lines.clone(), Color32::RED).ui(ui);
        });
    }
}

fn main() -> eframe::Result {
    // let (param_send, param_receive) = channel();
    // let (gate_send, gate_receive) = channel();
    //
    // let mut granny = Granulizer::new("handpan.wav", param_receive, gate_receive);
    // granny.init();
    // let sample_len = granny.buffer_size();
    //
    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let sink = Sink::try_new(&stream_handle).unwrap();

    // sink.append(granny);
    //
    // sink.sleep_until_end();

    let mut system = LSystem::new('0', vec!["1->11", "0->1[0]0"]);

    system.iterate(9);

    let lines = system
        .lines()
        .iter_mut()
        .map(|vec| {
            vec.iter()
                .map(|point| pos2(point.0, point.1))
                .collect::<Vec<_>>()
        })
        .collect();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Granular Plants",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, lines)))),
    )
}
