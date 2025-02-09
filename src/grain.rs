use crate::delay::{DelayParams, StereoDelay};
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
pub struct GranularEngine {
    path: PathBuf,
    samples: Vec<Frame>,
    grains: Vec<Grain>,
    params: GranularParams,
    param_rcvr: Receiver<GranularParams>,
    gate: bool,
    gate_rcvr: Receiver<bool>,
    delay: StereoDelay,
    sr: u32,
    spawn_timer: usize,
    scan: bool,
}

#[derive(Debug)]
pub struct Grain {
    t: usize,
    length: usize,
    start: usize,
    finished: bool,
}

#[derive(Debug, Clone)]
pub struct GranularParams {
    pub grain_density: usize,
    pub grain_length: usize,
    pub grain_spread: f32,
    pub start: usize,
    pub scan: Option<bool>,
    pub file: PathBuf,
}

impl Default for GranularParams {
    fn default() -> Self {
        Self {
            grain_density: 44000,
            grain_length: 44000,
            grain_spread: 0.0,
            start: 0,
            scan: None,
            file: PathBuf::from("handpan.wav"),
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

impl GranularEngine {
    pub fn new(
        path: &str,
        param_rcvr: Receiver<GranularParams>,
        gate_rcvr: Receiver<bool>,
        delay_rcvr: Receiver<DelayParams>,
    ) -> Self {
        Self {
            path: PathBuf::from(path),
            samples: vec![],
            grains: Vec::with_capacity(64), // Init with 64 grains
            params: Default::default(),
            param_rcvr,
            gate: true,
            gate_rcvr,
            delay: StereoDelay::new(2.45625, 1.53312, 44000, 0.2, 0.0, delay_rcvr),
            sr: 0,
            spawn_timer: 44000,
            scan: false,
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
            // Enable or disable scanning
            if let Some(scan) = params.scan {
                self.scan = scan;
            }
            self.params = params;
            println!("Granny received her params: \n{:?}", self.params);
        }
        if let Ok(gate) = self.gate_rcvr.try_recv() {
            self.gate = gate;
        }
    }

    pub fn buffer_size(&self) -> usize {
        self.samples.len()
    }
}

impl Iterator for GranularEngine {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.update_params();

        // Keep delay moving even when gate is not pressed
        let mut dry_sample = 0_f32;

        // Remove finished grains
        self.grains.retain(|grain| !grain.finished);
        // Spawn new grains if Gate is pressed
        if self.gate {
            if self.spawn_timer == 0 {
                let rand = (random::<f32>()
                    * self.params.grain_length as f32
                    * 2.0
                    * self.params.grain_spread) as usize;
                self.grains.push(Grain::new(
                    self.params.grain_length,
                    self.params.start + rand,
                ));
                self.spawn_timer = self.params.grain_density;
            }
            // Decrease timer
            self.spawn_timer -= 1;
            if self.scan {
                self.params.start += 1;
            }
        }
        // Read grains
        let fract = self.grains.len() as f32;
        for grain in &mut self.grains {
            let read_pos = grain.start + grain.t;
            let out = self.samples[read_pos % self.samples.len()].mono()
                * env(grain.length, grain.t, 0.5);

            grain.t += 1;
            if grain.t == grain.length {
                grain.finished = true;
            }
            dry_sample += out / fract.max(1.0);
        }

        let out = self.delay.process(dry_sample);
        Some(out)
    }
}

impl Source for GranularEngine {
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
