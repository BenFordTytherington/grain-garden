use crate::delay::DelayLine;
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
    delay: DelayLine,
    sr: u32,
}

#[derive(Debug)]
pub struct Grain {
    t: usize,
    params: GrainParams,
}

#[derive(Debug, Clone)]
pub struct GrainParams {
    pub length: usize,
    pub start: usize,
}

#[derive(Debug, Clone)]
pub struct GranulizerParams {
    pub grain_count: usize,
    pub grain_spread: f32,
    pub grain_params: Option<Vec<GrainParams>>,
    pub file: PathBuf,
}

impl Default for GrainParams {
    fn default() -> Self {
        Self {
            length: 44000,
            start: 0,
        }
    }
}

impl Default for GranulizerParams {
    fn default() -> Self {
        Self {
            grain_count: 1,
            grain_spread: 0.0,
            grain_params: Some(vec![GrainParams::default()]),
            file: PathBuf::from("juno.wav"),
        }
    }
}

impl Grain {
    pub fn new() -> Self {
        Self {
            t: 0,
            params: Default::default(),
        }
    }
}

impl GrainParams {
    pub fn random(len: usize) -> Self {
        Self {
            length: (random::<f32>() * 87000.0) as usize + 1000,
            start: (random::<f32>() * (len - 88000) as f32) as usize,
        }
    }
}

pub fn window(n: usize, t: usize) -> f32 {
    0.75 - 0.25 * (2.0 * PI * t as f32 / n as f32).cos()
}

impl Granulizer {
    pub fn new(path: &str, param_rcvr: Receiver<GranulizerParams>) -> Self {
        Self {
            path: PathBuf::from(path),
            samples: vec![],
            grains: (0..4).map(|_| Grain::new()).collect(), // Init with 4 grains
            params: Default::default(),
            param_rcvr,
            delay: DelayLine::new(44000, 44000 * 3),
            sr: 0,
        }
    }

    /// Will be used later when I add error propagation
    /// Initializes samples from path
    pub fn init(&mut self) {
        let reader = BufReader::new(File::open(&self.path).expect("Unknown file"));
        let decoder = Decoder::new_wav(reader).expect("Couldn't created decoder for file");

        self.sr = decoder.sample_rate();
        self.samples = decoder
            .convert_samples()
            .map(Frame::new)
            .collect();
    }

    pub fn update_params(&mut self) {
        if let Ok(params) = self.param_rcvr.try_recv() {
            if params.file != self.params.file {
                self.path = params.file.clone();
                self.init();
                println!("Initialised with new file");
            }
            self.params = params;
            // better way to do this without clone?
            if let Some(grain_params) = &self.params.grain_params {
                if grain_params.len() != self.grains.len() {
                    self.grains.resize_with(grain_params.len(), Grain::new);
                }
                for (index, params) in grain_params.iter().enumerate() {
                    self.grains[index].params = params.clone();
                }
            }
            println!("Granny received her params: \n{:?}", self.params);
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

        let mut dry_sample = 0_f32;
        for grain in &mut self.grains {
            let read_pos = grain.params.start + grain.t;
            let out = self.samples[read_pos].mono() * window(grain.params.length, grain.t);
            grain.t = (grain.t + 1) % grain.params.length;
            dry_sample += out;
        }
        let wet = self.delay.read();
        self.delay
            .write((dry_sample / self.grains.len() as f32) + (wet * 0.85)); // 75% Feedback
        self.delay.advance();
        Some((wet + dry_sample) * 0.5) // 50% mix
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
