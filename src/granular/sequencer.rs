use eframe::epaint::Pos2;
use rand::prelude::IndexedRandom;
use rand::rng;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct GrainMessage {
    // The start as a percentage of a total length, in this version, the whole sample
    pub start: f32,
    pub pan: f32,
}

#[derive(Debug)]
pub struct Sequencer {
    points: Vec<Pos2>, // Untransformed points
    max_height: f32,
    pub rate: f32,
    timer: usize,
    points_receiver: Receiver<Vec<Pos2>>,
    grain_events: Vec<GrainMessage>,
    index: usize,
}

impl Sequencer {
    pub fn new(points: Vec<Pos2>, rate: f32, rcvr: Receiver<Vec<Pos2>>) -> Self {
        Self {
            points,
            max_height: 0.0,
            rate,
            timer: 0,
            points_receiver: rcvr,
            grain_events: vec![],
            index: 0,
        }
    }

    pub fn get_events(&mut self) -> Vec<GrainMessage> {
        self.grain_events
            .drain(0..self.grain_events.len())
            .collect()
    }

    pub fn update_points(&mut self) {
        if let Ok(points) = self.points_receiver.try_recv() {
            self.max_height = points
                .iter()
                .map(|p| p.y)
                .max_by(|p1, p2| p1.total_cmp(p2))
                .unwrap_or(0.0);
            self.points = points
        }
    }

    pub fn update(&mut self) {
        self.update_points();
        if self.timer == 0 {
            self.trigger();
            self.timer = (44000.0 / self.rate) as usize
        }
        // Clamp to make sure timer resets with rate increasing
        self.timer = self.timer.clamp(0, (44000.0 / self.rate) as usize);
        self.timer -= 1;
    }

    /// At max should be 250, as this is half the window size
    /// Smaller values make the pan effect more pronounced on narrower trees
    const PAN_SENSITIVITY: f32 = 180.0;

    pub fn trigger(&mut self) {
        // if let Some(pos) = self.points.choose(&mut rng()) {
        //     let start = pos.y / self.max_height;
        //     let pan = (pos.x - 250.0) / 250.0;
        //     let msg = GrainMessage { start, pan };
        //     self.grain_events.push(msg);
        // }
        let points_len = self.points.len();
        if points_len > 0 {
            self.index = (self.index + 1) % (points_len - 1);
            let pos = self.points[self.index.min(points_len - 1)];

            let start = pos.y / self.max_height;
            let pan = ((pos.x - Self::PAN_SENSITIVITY) / Self::PAN_SENSITIVITY).clamp(-1.0, 1.0);
            let msg = GrainMessage { start, pan };
            self.grain_events.push(msg);
        }
    }
}
