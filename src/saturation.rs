#[derive(Debug, Clone, PartialEq)]
pub enum SaturationMode {
    Tape,
    Tube,
    Transistor,
}

#[derive(Debug)]
pub struct Saturater {
    drive: f32,
    prev: f32,
    mode: SaturationMode,
}

impl Saturater {
    const GAIN_FACTOR: f32 = 2.5; // Constant for input output gain
    pub fn new(drive: f32, mode: SaturationMode) -> Self {
        Self {
            drive,
            prev: 0.0,
            mode,
        }
    }

    pub fn set_mode(&mut self, mode: &SaturationMode) {
        self.mode = (*mode).clone();
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let gained = sample * Self::GAIN_FACTOR;
        let processed = match self.mode {
            SaturationMode::Tape => {
                let base = (gained * self.drive).tanh();
                let freq_weight = 0.9 + 0.35 * gained.abs();
                let hysteresis = 0.2 * (gained - self.prev); // Models magnetic tape

                self.prev = base * freq_weight + hysteresis;
                self.prev
            }
            SaturationMode::Tube => (gained * self.drive).tanh(),
            SaturationMode::Transistor => {
                gained.signum() * (1.0 - (-gained.abs() * self.drive).exp())
            }
        };
        processed / Self::GAIN_FACTOR
    }
}
