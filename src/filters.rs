use std::f32::consts::PI;

#[derive(Debug)]
pub struct LPFilter {
    a0: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    sr: usize,
    y: f32,
    x: f32,
    y1: f32,
    y2: f32,
    x1: f32,
    x2: f32,
}

impl LPFilter {
    pub fn new(sr: usize, cutoff: f32) -> Self {
        let mut this = Self {
            a0: 0.0,
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            sr,
            y: 0.0,
            x: 0.0,
            y1: 0.0,
            y2: 0.0,
            x1: 0.0,
            x2: 0.0,
        };

        this.compute_coeffs(cutoff);

        this
    }

    pub fn compute_coeffs(&mut self, cutoff: f32) {
        let w = 2.0 * PI * cutoff / self.sr as f32;
        let alpha = w.sin() / 1.414; // Butterworth filter Q value, somewhat arbitrary
        let cw = w.cos();

        self.a0 = 1.0 + alpha;
        self.a1 = -2.0 * cw;
        self.a2 = 1.0 - alpha;
        self.b0 = (1.0 - cw) / 2.0;
        self.b1 = 1.0 - cw;
        self.b2 = (1.0 - cw) / 2.0;
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.x2 = self.x1;
        self.x1 = self.x;
        self.x = sample;
        self.y2 = self.y1;
        self.y1 = self.y;

        self.y = (self.b0 * self.x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2) / self.a0;

        self.y
    }
}