mod grain;
mod scheduler;

use crate::delay::{DelayParams, FeedbackParams, StereoDelay};
use crate::dsp::StereoFrame;
use crate::granular::grain::Grain;
use rand::random;
use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

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
            grains: (0..64).map(|_| Grain::finished()).collect(), // Init with 64 grains
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
            dry += grain.read(&self.samples);
        }

        self.delay.process(dry).scale(self.params.gain)
    }

    pub fn process_block(&mut self, buf: &mut [StereoFrame]) {
        for i in 0..buf.len() {
            buf[i] = self.process();
        }
    }
}
