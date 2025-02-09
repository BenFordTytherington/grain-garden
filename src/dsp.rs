#[derive(Debug, Clone)]
pub struct Frame(pub f32, pub f32);

impl Frame {
    pub fn new(sample: f32) -> Self {
        Self(sample, sample)
    }

    pub fn mono(&self) -> f32 {
        (self.1 + self.0) / 2.0
    }
}
