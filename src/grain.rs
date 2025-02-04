use crate::delay::StereoDelay;
use crate::dsp::Frame;
use rand::random;
use rodio::{Decoder, Source};
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

#[derive(Debug)]
pub struct Granulizer {
    path: PathBuf,
    samples: Vec<Frame>,
    grains: Vec<Grain>,
    params: GranulizerParams,
    param_rcvr: Receiver<GranulizerParams>,
    gate: bool,
    gate_rcvr: Receiver<bool>,
    delay: StereoDelay,
    sr: u32,
}

#[derive(Debug)]
pub struct Grain {
    t: usize,
    length: usize,
    start: usize,
    finished: bool,
}

#[derive(Debug, Clone)]
pub struct GranulizerParams {
    pub grain_count: usize,
    pub grain_length: usize,
    pub grain_spread: f32,
    pub start: usize,
    pub file: PathBuf,
}

impl Default for GranulizerParams {
    fn default() -> Self {
        Self {
            grain_count: 64,
            grain_length: 44000,
            grain_spread: 0.0,
            start: 0,
            file: PathBuf::from("chopin.wav"),
        }
    }
}

impl Grain {
    pub fn new(length: usize, start: usize) -> Self {
        Self {
            t: 0,
            length,
            start,
            finished: false,
        }
    }

    pub fn reinit(&mut self, length: usize, start: usize) {
        println!("Reinit a grain with: \n   l: {:?}, s: {:?}", length, start);
        self.t = 0;
        self.finished = false;
        self.start = start;
        self.length = length;
    }
}

impl Default for Grain {
    fn default() -> Self {
        Self {
            t: 0,
            length: 44000,
            start: 0,
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
    if t <= m {
        ((-c1 * t).exp() - 1.0) / ((-c1 * m).exp() - 1.0)
    } else {
        ((c2 * (t - 1.0)).exp() - 1.0) / ((c2 * (m - 1.0)).exp() - 1.0)
    }
}

pub fn env(n: usize, t: usize, m: f32) -> f32 {
    let t_norm = t as f32 / n as f32;
    // ad(t_norm, m)
    exp(t_norm, m, 25.0, -6.0)
}

impl Granulizer {
    pub fn new(
        path: &str,
        param_rcvr: Receiver<GranulizerParams>,
        gate_rcvr: Receiver<bool>,
    ) -> Self {
        Self {
            path: PathBuf::from(path),
            samples: vec![],
            grains: (0..32).map(|_| Grain::default()).collect(), // Init with 64 grains
            params: Default::default(),
            param_rcvr,
            gate: false,
            gate_rcvr,
            delay: StereoDelay::new(1.45625, 0.53312, 44000, 0.5, 0.4),
            sr: 0,
        }
    }

    /// Will be used later when I add error propagation
    /// Initializes samples from path
    pub fn init(&mut self) {
        let reader = BufReader::new(File::open(&self.path).expect("Unknown file"));
        let decoder = Decoder::new_wav(reader).expect("Couldn't created decoder for file");

        self.sr = decoder.sample_rate();
        self.samples = decoder.convert_samples().map(Frame::new).collect();
    }

    pub fn update_params(&mut self) {
        if let Ok(params) = self.param_rcvr.try_recv() {
            if params.file != self.params.file {
                self.path = params.file.clone();
                self.init();
                println!("Initialised with new file");
            }
            self.params = params;
            println!("Granny received her params: \n{:?}", self.params);
        }
        if let Ok(true) = self.gate_rcvr.try_recv() {
            self.gate = !self.gate;
            if self.gate {
                println!("Gate on");
            } else {
                println!("Gate off")
            }
        }
    }

    pub fn buffer_size(&self) -> usize {
        self.samples.len()
    }
}

impl Iterator for Granulizer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.update_params();

        // Keep delay moving even when gate is not pressed
        let mut dry_sample = 0_f32;
        if self.gate {
            for grain in &mut self.grains {
                // Respawn grain
                if grain.finished {
                    let start_rand = (random::<f32>() * 2000.0) as usize;
                    grain.reinit(self.params.grain_length, self.params.start + start_rand)
                }
                let read_pos = grain.start + grain.t;
                let out = self.samples[read_pos % self.samples.len()].mono()
                    // * window(grain.length, grain.t);
                    * env(grain.length, grain.t, 0.2);
                grain.t += 1;
                if grain.t == grain.length {
                    grain.finished = true;
                }
                dry_sample += out;
            }
        }
        let out = self.delay.process(dry_sample / self.grains.len() as f32);
        Some(out)
    }
}

impl Source for Granulizer {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples.len())
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sr
    }

    fn total_duration(&self) -> Option<Duration> {
        self.current_frame_len()
            .map(|samples| Duration::from_secs(samples as u64 * self.sr as u64))
    }
}
