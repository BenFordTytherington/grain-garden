use floem::prelude::dropdown::Dropdown;
use floem::prelude::slider::Slider;
use floem::prelude::*;
use rand::random;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Debug)]
struct Granulizer {
    path: PathBuf,
    samples: Vec<f32>,
    params: GrainParams,
    param_rcvr: Receiver<GrainParams>,
    t: usize,
    sr: u32,
}

#[derive(Debug, Clone)]
struct GrainParams {
    length: usize,
    start: usize,
    file: PathBuf,
}

impl Default for GrainParams {
    fn default() -> Self {
        Self {
            length: 44000,
            start: 0,
            file: PathBuf::from("handpan.wav"),
        }
    }
}

pub fn window(n: usize, t: usize) -> f32 {
    0.75 - 0.25 * (2.0 * PI * t as f32 / n as f32).cos()
}

impl Granulizer {
    pub fn new(path: &str, param_rcvr: Receiver<GrainParams>) -> Self {
        Self {
            path: PathBuf::from(path),
            samples: vec![],
            params: Default::default(),
            param_rcvr,
            t: 0,
            sr: 0,
        }
    }

    /// Will be used later when I add error propagation
    /// Initializes samples from path
    pub fn init(&mut self) {
        let reader = BufReader::new(File::open(&self.path).expect("Unknown file"));
        let decoder = Decoder::new_wav(reader).expect("Couldn't created decoder for file");

        self.sr = decoder.sample_rate();
        self.samples = decoder.convert_samples().collect();
    }
}

impl Iterator for Granulizer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(params) = self.param_rcvr.try_recv() {
            if params.file != self.params.file {
                self.path = params.file.clone();
                self.init();
                println!("Initialised with new file");
            }
            self.params = params;
            println!("Granny received her params: \n{:?}", self.params);
        }
        let read_pos = self.params.start + self.t;
        let out = Some(self.samples[read_pos] * window(self.params.length, self.t));
        self.t = (self.t + 1) % self.params.length;
        out
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

// In future could split components to own function, where I can then do the clone of sender,
// and then call all of them in an app "main" function
fn app(sender: Sender<GrainParams>, len: usize) -> impl IntoView {
    let start = RwSignal::new(0.0.pct());
    let length = RwSignal::new(0.5.pct());

    let params = RwSignal::new(GrainParams::default());

    let filename = RwSignal::new("rhodes.wav");

    let button_send = sender.clone();
    let start_send = sender.clone();
    let length_send = sender.clone();
    let file_send = sender.clone();

    h_stack((
        button("Randomise").action(move || {
            let new_params = GrainParams {
                length: (random::<f32>() * 87000.0) as usize + 1000,
                start: (random::<f32>() * (len - 88000) as f32) as usize,
                file: PathBuf::from("rhodes.wav"),
            };
            params.set(new_params.clone()); // Could be a better way to set this
            button_send.send(new_params).expect("Failed to send params");
        }),
        v_stack((
            Dropdown::custom(
                move || filename.get(),
                |main_item| text(main_item).into_any(),
                vec!["handpan.wav", "juno.wav", "rhodes.wav"],
                |list_item| text(list_item).into_any(),
            )
            .on_accept(move |item| {
                filename.set(item);
                let mut new_params = params.get();
                new_params.file = PathBuf::from(item);
                params.set(new_params.clone());
                file_send.send(new_params).expect("Failed to send params");
            }),
            Slider::new(move || start.get()).on_change_pct(move |value| {
                let mut new_params = params.get();
                let max_len = len - new_params.length - 2;
                new_params.start = (value.0 * max_len as f64 / 100.0) as usize;
                start.set(value);
                params.set(new_params.clone());
                start_send.send(new_params).expect("Failed to send params");
            }),
            Slider::new(move || length.get()).on_change_pct(move |value| {
                let mut new_params = params.get();
                new_params.length = ((value.0 * 88000.0 / 100.0) as usize).max(1); // ensure it can't be 0
                length.set(value);
                params.set(new_params.clone());
                length_send.send(new_params).expect("Failed to send params");
            }),
        )),
    ))
}

fn main() {
    let (sender, receiver) = channel();

    let mut granny = Granulizer::new("rhodes.wav", receiver);
    granny.init();
    let sample_len = granny.samples.len();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(granny);

    floem::launch(move || app(sender, sample_len));

    sink.sleep_until_end();
}
