use crate::dsp::StereoFrame;
use std::f32::consts::PI;

#[derive(Debug)]
pub struct Grain {
    t: usize,
    length: usize,
    start: usize,
    pan: f32,
    pub finished: bool,
}

impl Grain {
    pub fn new(length: usize, start: usize, pan: f32) -> Self {
        Self {
            t: 0,
            length,
            start,
            pan,
            finished: false,
        }
    }

    pub fn finished() -> Self {
        Self {
            finished: true,
            ..Default::default()
        }
    }

    pub fn read(&mut self, buffer: &[StereoFrame]) -> StereoFrame {
        let read_pos = self.start + self.t;
        let out = buffer[read_pos % buffer.len()];

        self.t += 2;
        if self.t >= self.length {
            self.finished = true;
        };
        let windowed = out.scale(window(self.length, self.t));
        StereoFrame(
            (1.0 - self.pan) * windowed.0 * 0.5,
            (1.0 + self.pan) * windowed.1 * 0.5,
        )
    }
}

impl Default for Grain {
    fn default() -> Self {
        Self {
            t: 0,
            length: 44000,
            start: 0,
            pan: 0.0,
            finished: false,
        }
    }
}

pub fn window(n: usize, t: usize) -> f32 {
    0.5 - 0.5 * (2.0 * PI * t as f32 / n as f32).cos()
}

// linear Envelope with t from 0 to 1
pub fn ad(t: f32, m: f32) -> f32 {
    if t <= m {
        t / m
    } else {
        (t - 1.0) / (m - 1.0)
    }
}

pub fn exp(t: f32, m: f32, c1: f32, c2: f32) -> f32 {
    // Sub in linear envelope if coeffs are too small
    if (c1.abs() <= 0.01) | (c2.abs() <= 0.01) {
        ad(t, m)
    } else if t <= m {
        ((-c1 * t).exp() - 1.0) / ((-c1 * m).exp() - 1.0)
    } else {
        ((c2 * (t - 1.0)).exp() - 1.0) / ((c2 * (m - 1.0)).exp() - 1.0)
    }
}

pub fn env(n: usize, t: usize, m: f32) -> f32 {
    let t_norm = t as f32 / n as f32;
    exp(t_norm, m, -5.0, -5.0)
}
