use crate::dsp::StereoFrame;
use crate::saturation::{Saturater, SaturationMode};
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
    sat_l: Saturater,
    sat_r: Saturater,
    sr: usize,
    params: DelayParams,
    params_receiver: Receiver<DelayParams>,
    feedback_params: FeedbackParams, // Parameters for the effects inside the fb loop
    feedback_params_receiver: Receiver<FeedbackParams>,
}

impl StereoDelay {
    pub fn new(
        time_l: f32,
        time_r: f32,
        sr: usize,
        feedback: f32,
        mix: f32,
        params_receiver: Receiver<DelayParams>,
        fb_receiver: Receiver<FeedbackParams>,
    ) -> Self {
        let time_samples_l = (time_l * sr as f32) as usize;
        let time_samples_r = (time_r * sr as f32) as usize;
        Self {
            dl_left: DelayLine::new(time_samples_l, 44000 * 6),
            dl_right: DelayLine::new(time_samples_r, 44000 * 6),
            sat_l: Saturater::new(0.7, SaturationMode::Tape),
            sat_r: Saturater::new(0.7, SaturationMode::Tape),
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
            feedback_params: Default::default(),
            feedback_params_receiver: fb_receiver,
        }
    }

    pub fn update_params(&mut self) {
        if let Ok(params) = self.params_receiver.try_recv() {
            println!("Delay received params: \n{:?}", self.params);
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
        if let Ok(params) = self.feedback_params_receiver.try_recv() {
            if self.feedback_params.mode != params.mode {
                self.sat_l.set_mode(&params.mode);
                self.sat_r.set_mode(&params.mode);
            }
            self.feedback_params = params;
        }
    }

    pub fn process(&mut self, sample: StereoFrame) -> StereoFrame {
        self.update_params();

        if !self.params.bypass {
            // Process read, including saturation
            let mut wet_l = self.dl_left.read();
            let mut wet_r = self.dl_right.read();
            if self.feedback_params.saturate {
                // Scaling to ensure feedback doesn't exceed its limit
                wet_l = 0.99 * self.sat_l.process(wet_l);
                wet_r = 0.99 * self.sat_r.process(wet_r);
            }

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

#[derive(Debug, Clone)]
pub struct FeedbackParams {
    pub drive: f32,
    pub saturate: bool,
    pub mode: SaturationMode,
}

impl Default for FeedbackParams {
    fn default() -> Self {
        Self {
            drive: 0.7,
            saturate: false,
            mode: SaturationMode::Tape,
        }
    }
}
