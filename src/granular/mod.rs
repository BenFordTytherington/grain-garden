pub mod grain;
pub mod sequencer;

use crate::dsp::StereoFrame;
use eframe::emath::Pos2;
use grain::Grain;
use rodio::{Decoder, Source};
use sequencer::Sequencer;
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
    sr: u32,
    scan: bool,
    seq: Sequencer,
}

#[derive(Debug, Clone)]
pub struct GranularParams {
    pub grain_length: usize, // Length grains will play for
    pub grain_spread: usize, // Window size in samples that grains can spawn in
    pub gain: f32,
    pub start: usize,
    pub scan: Option<bool>,
    pub file: PathBuf,
    pub density: f32, // How often grains will be spawned, in hz
}

impl Default for GranularParams {
    fn default() -> Self {
        Self {
            grain_length: 44000,
            grain_spread: 88000,
            gain: 0.7,
            start: 0,
            scan: None,
            file: PathBuf::from("assets/audio/handpan_trimmed.wav"),
            density: 1.0,
        }
    }
}

impl GranularEngine {
    pub fn new(
        path: PathBuf,
        param_rcvr: Receiver<GranularParams>,
        gate_rcvr: Receiver<bool>,
        seq_rcvr: Receiver<Vec<Pos2>>,
    ) -> Self {
        Self {
            path,
            samples: vec![],
            grains: (0..64).map(|_| Grain::finished()).collect(), // Init with 64 grains
            params: Default::default(),
            param_rcvr,
            gate: true,
            gate_rcvr,
            sr: 0,
            scan: false,
            seq: Sequencer::new(vec![], 1.0, seq_rcvr),
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
            if params.density != self.seq.rate {
                self.seq.rate = params.density;
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

    pub fn spawn_grain_at(&mut self, start: usize, pan: f32) {
        self.grains
            .push(Grain::new(self.params.grain_length, start, pan));
    }

    // Return one frame of granular audio
    pub fn process(&mut self) -> StereoFrame {
        self.update_params();
        self.seq.update();

        // Keep delay processing even when gate is not pressed
        let mut dry = StereoFrame(0.0, 0.0);

        // Remove finished grains
        self.grains.retain(|grain| !grain.finished);

        // Spawn new grains if Gate is pressed
        if self.gate {
            for msg in self.seq.get_events() {
                let start =
                    self.params.start + (msg.start * self.params.grain_spread as f32) as usize;
                self.spawn_grain_at(start, msg.pan);
            }

            if self.scan {
                self.params.start += 1;
            }
        }

        // Read grains even if gate is not pressed, for smooth decay
        for grain in &mut self.grains {
            dry += grain.read(&self.samples);
        }

        dry.scale(self.params.gain * 1.2 / self.grains.len().max(1) as f32)
    }

    pub fn process_block(&mut self, buf: &mut [StereoFrame]) {
        for i in 0..buf.len() {
            buf[i] = self.process();
        }
    }
}
