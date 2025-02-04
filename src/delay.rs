#[derive(Debug)]
pub struct DelayLine {
    buf_size: usize,
    samples: Vec<f32>,
    read_ptr: usize,
    time: usize,
}

impl DelayLine {
    pub fn new(time: usize, buf_size: usize) -> Self {
        Self {
            buf_size,
            samples: vec![0.0; buf_size],
            read_ptr: 0,
            time,
        }
    }

    pub fn write(&mut self, sample: f32) {
        let write_ptr = (self.read_ptr + self.time) % self.buf_size;
        self.samples[write_ptr] = sample;
    }

    pub fn read(&self) -> f32 {
        self.samples[self.read_ptr]
    }

    pub fn advance(&mut self) {
        self.read_ptr = (self.read_ptr + 1) % self.buf_size;
    }
}

#[derive(Debug)]
pub struct StereoDelay {
    dl_left: DelayLine,
    dl_right: DelayLine,
    sr: usize,
    feedback: f32,
    mix: f32,
    time_l: f32,
    time_r: f32,
    channel: bool, // Bool used to keep track of which delay line to write to
}

impl StereoDelay {
    pub fn new(time_l: f32, time_r: f32, sr: usize, feedback: f32, mix: f32) -> Self {
        let time_samples_l = (time_l * sr as f32) as usize;
        let time_samples_r = (time_r * sr as f32) as usize;
        Self {
            dl_left: DelayLine::new(time_samples_l, time_samples_l * 2),
            dl_right: DelayLine::new(time_samples_r, time_samples_r * 2),
            sr,
            feedback,
            mix,
            time_l,
            time_r,
            channel: false,
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let dl = if self.channel {
            &mut self.dl_right
        } else {
            &mut self.dl_left
        };
        let wet = dl.read();

        dl.write(sample + (wet * self.feedback));
        dl.advance();

        self.channel = !self.channel;
        sample * (1.0 - self.mix) + wet * self.mix
    }
}
