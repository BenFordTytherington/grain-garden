mod delay;
mod dsp;
mod grain;

use crate::grain::{GrainParams, Granulizer, GranulizerParams};
use floem::prelude::dropdown::Dropdown;
use floem::prelude::*;
use rand::random;
use rodio::{OutputStream, Sink};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use floem::event::{Event, EventListener};
use floem::keyboard::Key;
use floem::keyboard::NamedKey::Space;

// In future could split components to own function, where I can then do the clone of sender,
// and then call all of them in an app "main" function
fn app(sender: Sender<GranulizerParams>, gate: Sender<bool>, len: usize) -> impl IntoView {
    // let start = RwSignal::new(0.0.pct());
    // let length = RwSignal::new(0.5.pct());

    let params = RwSignal::new(GranulizerParams::default());

    let filename = RwSignal::new("juno.wav");

    let button_send = sender.clone();
    // let start_send = sender.clone();
    // let length_send = sender.clone();
    let file_send = sender.clone();

    let view = h_stack((
        button("Randomise").action(move || {
            let grain_count = (random::<f32>() * 7.0) as usize + 1;
            let grain_params = (0..grain_count).map(|_| GrainParams::random(len)).collect();
            let new_params = GranulizerParams {
                grain_count,
                grain_spread: random::<f32>(),
                grain_params: Some(grain_params),
                file: PathBuf::from("juno.wav"),
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
            // Slider::new(move || start.get()).on_change_pct(move |value| {
            //     let mut new_params = params.get();
            //     let max_len = len - new_params.length - 2;
            //     new_params.start = (value.0 * max_len as f64 / 100.0) as usize;
            //     start.set(value);
            //     params.set(new_params.clone());
            //     start_send.send(new_params).expect("Failed to send params");
            // }),
            // Slider::new(move || length.get()).on_change_pct(move |value| {
            //     let mut new_params = params.get();
            //     new_params.length = ((value.0 * 88000.0 / 100.0) as usize).max(1); // ensure it can't be 0
            //     length.set(value);
            //     params.set(new_params.clone());
            //     length_send.send(new_params).expect("Failed to send params");
            // }),
        )),
    ));
    view.on_event_stop(EventListener::KeyDown, move |e| {
        if let Event::KeyDown(e) = e {
            if e.key.logical_key == Key::Named(Space) {
                gate.send(true).expect("Failed to send Gate");
            }
        }
    })
}

fn main() {
    let (param_send, param_receive) = channel();
    let (gate_send, gate_receive) = channel();

    let mut granny = Granulizer::new("juno.wav", param_receive, gate_receive);
    granny.init();
    let sample_len = granny.buffer_size();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(granny);

    floem::launch(move || app(param_send, gate_send, sample_len));

    sink.sleep_until_end();
}
