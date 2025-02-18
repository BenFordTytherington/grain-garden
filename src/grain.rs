use crate::delay::{DelayParams, FeedbackParams, StereoDelay};
use crate::dsp::StereoFrame;
use rand::random;
use rodio::{Decoder, Source};
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

// TODO Granular engine now returns a buffer of samples
// No need for receivers anymore?? as it can be directly modified by GUI in main thread
// Delay needs to be extracted
#[derive(Debug)]
pub struct GranularEngine {
    path: PathBuf,
    samples: Vec<StereoFrame>,
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
    pan: f32,
    finished: bool,
}

#[derive(Debug, Clone)]
pub struct GranularParams {
    pub grain_density: usize,
    pub grain_length: usize,
    pub grain_spread: f32,
    pub pan_spread: f32,
    pub gain: f32,
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
            pan_spread: 0.0,
            gain: 0.7,
            start: 0,
            scan: None,
            file: PathBuf::from("assets/audio/handpan.wav"),
        }
    }
}

impl Grain {
    pub fn new(length: usize, start: usize, pan: f32) -> Self {
        Self {
            t: 0,
            length,
            start,
            pan,
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
            pan: 0.0,
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
        fb_rcvr: Receiver<FeedbackParams>,
    ) -> Self {
        Self {
            path: PathBuf::from(path),
            samples: vec![],
            grains: Vec::with_capacity(64), // Init with 64 grains
            params: Default::default(),
            param_rcvr,
            gate: true,
            gate_rcvr,
            delay: StereoDelay::new(2.45625, 1.53312, 44000, 0.2, 0.5, delay_rcvr, fb_rcvr),
            sr: 0,
            spawn_timer: 44000,
            scan: false,
        }
    }

    /// Initializes samples from path
    pub fn init(&mut self) {
        let reader = BufReader::new(File::open(&self.path).expect("Unknown file"));
        let decoder = Decoder::new_wav(reader).expect("Couldn't created decoder for file");

        self.sr = decoder.sample_rate();
        self.samples = decoder.convert_samples().map(StereoFrame::new).collect();
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

    pub fn spawn_grain(&mut self) {
        let start_rand =
            (random::<f32>() * self.params.grain_length as f32 * 2.0 * self.params.grain_spread)
                as usize;

        let pan_rand = (random::<f32>() * 2.0 * self.params.pan_spread) - self.params.pan_spread;
        self.grains.push(Grain::new(
            self.params.grain_length,
            self.params.start + start_rand,
            pan_rand,
        ));
        self.spawn_timer = self.params.grain_density;
    }

    // Return one frame of granular audio
    pub fn process(&mut self) -> StereoFrame {
        self.update_params();

        // Keep delay processing even when gate is not pressed
        let mut dry = StereoFrame(0.0, 0.0);

        // Remove finished grains
        self.grains.retain(|grain| !grain.finished);

        // Spawn new grains if Gate is pressed
        if self.gate {
            // Decrease timer
            self.spawn_timer -= 1;

            if self.spawn_timer == 0 {
                self.spawn_grain();
            }
            if self.scan {
                self.params.start += 1;
            }
        }

        // Read grains even if gate is not pressed, for smooth decay
        for grain in &mut self.grains {
            let read_pos = grain.start + grain.t;
            let out = &self.samples[read_pos % self.samples.len()];

            let pan = grain.pan;

            grain.t += 2;
            if grain.t >= grain.length {
                grain.finished = true;
            };
            let windowed = out.scale(window(grain.length, grain.t));
            dry += StereoFrame(
                (1.0 - pan) * windowed.0 * 0.5,
                (1.0 + pan) * windowed.1 * 0.5,
            );
        }

        self.delay.process(dry).scale(self.params.gain)
    }

    pub fn process_block(&mut self, buf: &mut [StereoFrame]) {
        for i in 0..buf.len() {
            buf[i] = self.process();
        }
    }
}
