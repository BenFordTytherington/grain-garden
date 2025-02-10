use std::ops::AddAssign;

#[derive(Debug, Clone, Copy)]
pub struct StereoFrame(pub f32, pub f32);

impl StereoFrame {
    pub fn new(sample: f32) -> Self {
        Self(sample, sample)
    }

    pub fn mono(&self) -> f32 {
        (self.1 + self.0) / 2.0
    }

    pub fn scale(&self, scale: f32) -> Self {
        Self(self.0 * scale, self.1 * scale)
    }
}

impl AddAssign for StereoFrame {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

pub fn interleave(buf: &[StereoFrame]) -> Vec<f32> {
    buf.iter().flat_map(|frame| [frame.0, frame.1]).collect()
}
