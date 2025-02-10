use crate::dsp::Frame;
use std::sync::mpsc::Receiver;

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

    pub fn set_time(&mut self, time: usize) {
        self.time = time;
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
    params: DelayParams,
    params_receiver: Receiver<DelayParams>,
}

impl StereoDelay {
    pub fn new(
        time_l: f32,
        time_r: f32,
        sr: usize,
        feedback: f32,
        mix: f32,
        params_receiver: Receiver<DelayParams>,
    ) -> Self {
        let time_samples_l = (time_l * sr as f32) as usize;
        let time_samples_r = (time_r * sr as f32) as usize;
        Self {
            dl_left: DelayLine::new(time_samples_l, 44000 * 6),
            dl_right: DelayLine::new(time_samples_r, 44000 * 6),
            sr,
            params: DelayParams {
                feedback,
                mix,
                time_l,
                time_r,
            },
            params_receiver,
        }
    }

    pub fn update_params(&mut self) {
        if let Ok(params) = self.params_receiver.try_recv() {
            self.params = params;
            self.dl_left
                .set_time((self.params.time_l * self.sr as f32) as usize);
            self.dl_right
                .set_time((self.params.time_r * self.sr as f32) as usize);
        }
    }

    pub fn process(&mut self, sample: Frame) -> Frame {
        self.update_params();

        let wet_l = self.dl_left.read();
        let wet_r = self.dl_right.read();

        self.dl_left
            .write(sample.0 + (wet_l * self.params.feedback));
        self.dl_right
            .write(sample.1 + (wet_r * self.params.feedback));

        self.dl_left.advance();
        self.dl_right.advance();

        let mixed = |dry, wet| dry * (1.0 - self.params.mix) + wet * self.params.mix;
        Frame(mixed(sample.0, wet_l), mixed(sample.1, wet_r))
    }
}

#[derive(Debug, Clone)]
pub struct DelayParams {
    pub feedback: f32,
    pub mix: f32,
    pub time_l: f32,
    pub time_r: f32,
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            feedback: 0.5,
            mix: 0.2,
            time_l: 1.0,
            time_r: 2.0,
        }
    }
}
