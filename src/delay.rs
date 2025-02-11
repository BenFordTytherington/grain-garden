use crate::dsp::StereoFrame;
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct DelayLine {
    buf_size: usize,
    samples: Vec<f32>,
    write_ptr: usize,
    current_time: f32, // Times can be fractional, for smoother interpolation
    target_time: f32,
}

impl DelayLine {
    pub fn new(time: usize, buf_size: usize) -> Self {
        Self {
            buf_size,
            samples: vec![0.0; buf_size],
            write_ptr: 0,
            current_time: time as f32,
            target_time: time as f32,
        }
    }

    pub fn set_time_smooth(&mut self, time: usize) {
        self.target_time = time as f32;
    }

    pub fn set_time(&mut self, time: usize) {
        self.current_time = time as f32;
    }

    pub fn write(&mut self, sample: f32) {
        self.samples[self.write_ptr] = sample;
    }

    pub fn read(&self) -> f32 {
        let mut read_ptr = self.write_ptr as f32 - self.current_time;
        if read_ptr < 0.0 {
            read_ptr += self.buf_size as f32;
        }
        // Interpolate samples
        let index = read_ptr.floor() as usize % self.buf_size;
        let first = self.samples[index];
        let second = self.samples[(index + 1) % self.buf_size];
        let t = read_ptr.fract();
        (1.0 - t) * first + t * second
    }

    pub fn advance(&mut self) {
        // Smoothly interpolate delay times
        if self.current_time > self.target_time {
            self.current_time -= 0.5;
        } else if self.current_time < self.target_time {
            self.current_time += 0.5;
        }
        self.write_ptr = (self.write_ptr + 1) % self.buf_size;
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
                bypass: true,
                pitch: false,
            },
            params_receiver,
        }
    }

    pub fn update_params(&mut self) {
        if let Ok(params) = self.params_receiver.try_recv() {
            self.params = params;
            let l_time = (self.params.time_l * self.sr as f32) as usize;
            let r_time = (self.params.time_r * self.sr as f32) as usize;
            if self.params.pitch {
                self.dl_left.set_time_smooth(l_time);
                self.dl_right.set_time_smooth(r_time);
            } else {
                self.dl_left.set_time(l_time);
                self.dl_right.set_time(r_time);
            }
        }
    }

    pub fn process(&mut self, sample: StereoFrame) -> StereoFrame {
        self.update_params();

        if !self.params.bypass {
            let wet_l = self.dl_left.read();
            let wet_r = self.dl_right.read();

            self.dl_left
                .write(sample.0 + (wet_l * self.params.feedback));
            self.dl_right
                .write(sample.1 + (wet_r * self.params.feedback));

            self.dl_left.advance();
            self.dl_right.advance();

            let mixed = |dry, wet| dry * (1.0 - self.params.mix) + wet * self.params.mix;
            StereoFrame(mixed(sample.0, wet_l), mixed(sample.1, wet_r))
        } else {
            sample
        }
    }
}

#[derive(Debug, Clone)]
pub struct DelayParams {
    pub feedback: f32,
    pub mix: f32,
    pub time_l: f32,
    pub time_r: f32,
    pub bypass: bool,
    pub pitch: bool,
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            feedback: 0.5,
            mix: 0.2,
            time_l: 1.0,
            time_r: 2.0,
            bypass: true,
            pitch: false,
        }
    }
}
