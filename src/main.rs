mod delay;
mod dsp;
mod grain;
mod lsystem;

use std::collections::HashMap;
use crate::grain::{Granulizer, GranulizerParams};
use floem::event::{Event, EventListener};
use floem::keyboard::Key;
use floem::keyboard::NamedKey::Space;
use floem::prelude::dropdown::Dropdown;
use floem::prelude::slider::Slider;
use floem::prelude::*;
use rand::random;
use rodio::{OutputStream, Sink};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use crate::lsystem::LSystem;

// In future could split components to own function, where I can then do the clone of sender,
// and then call all of them in an app "main" function
fn app(sender: Sender<GranulizerParams>, gate: Sender<bool>, len: usize) -> impl IntoView {
    let start = RwSignal::new(0.0.pct());
    let length = RwSignal::new(0.5.pct());
    let spread = RwSignal::new(0.5.pct());
    let scan = RwSignal::new(false);

    let params = RwSignal::new(GranulizerParams::default());

    let filename = RwSignal::new("handpan.wav");

    let button_send = sender.clone();
    let start_send = sender.clone();
    let length_send = sender.clone();
    let file_send = sender.clone();
    let grain_up_send = sender.clone();
    let grain_down_send = sender.clone();
    let spread_send = sender.clone();

    let view = (
        button("Scan").action(move || {
            let mut new_params = params.get();
            let current = scan.get();
            new_params.scan = Some(!current);
            scan.set(!current);
            params.set(new_params.clone()); // Could be a better way to set this
            button_send.send(new_params).expect("Failed to send params");
        }).style(|s| s.width(100)),
        h_stack((
            button("-").action(move || {
                let mut new_params = params.get();
                new_params.grain_density -= (new_params.grain_length as f32 / 64.0) as usize;
                params.set(new_params.clone());
                grain_down_send.send(new_params).expect("Failed to send params");
            }),
            button("+").action(move || {
                let mut new_params = params.get();
                new_params.grain_density += (new_params.grain_length as f32 / 64.0) as usize;
                params.set(new_params.clone());
                grain_up_send.send(new_params).expect("Failed to send params");
            }),
        )),
        Dropdown::custom(
            move || filename.get(),
            |main_item| text(main_item).into_any(),
            vec!["handpan.wav", "juno.wav", "rhodes.wav", "chopin.wav"],
            |list_item| text(list_item).into_any(),
        ).on_accept(move |item| {
            filename.set(item);
            let mut new_params = params.get();
            new_params.file = PathBuf::from(item);
            params.set(new_params.clone());
            file_send.send(new_params).expect("Failed to send params");
        }).style(|s| s.width(100)),
        "Start",
        Slider::new(move || start.get()).on_change_pct(move |value| {
            let mut new_params = params.get();
            new_params.start = (value.0 * len as f64 / 100.0) as usize;
            start.set(value);
            params.set(new_params.clone());
            start_send.send(new_params).expect("Failed to send params");
        }).style(|s| s.width_full()),
        "Length",
        Slider::new(move || length.get()).on_change_pct(move |value| {
            let mut new_params = params.get();
            new_params.grain_length = (value.0 * 131000.0 / 100.0) as usize + 1000; // ensure it can't be 0
            length.set(value);
            params.set(new_params.clone());
            length_send.send(new_params).expect("Failed to send params");
        }).style(|s| s.width_full()),
        "Spread",
        Slider::new(move || spread.get()).on_change_pct(move |value| {
            let mut new_params = params.get();
            new_params.grain_spread = value.0 as f32 / 100.0;
            spread.set(value);
            params.set(new_params.clone());
            spread_send.send(new_params).expect("Failed to send params");
        }).style(|s| s.width_full())
    ).style(|s| s.flex_col().gap(6).items_center().width_full());

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

    let mut granny = Granulizer::new("handpan.wav", param_receive, gate_receive);
    granny.init();
    let sample_len = granny.buffer_size();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // sink.append(granny);
    //
    // floem::launch(move || app(param_send, gate_send, sample_len));
    //
    // sink.sleep_until_end();

    let mut rules = HashMap::new();
    rules.insert('1', "11".to_string());
    rules.insert('0', "1[0]0".to_string());

    let mut system = LSystem {
        axiom: "0".to_string(),
        result: "0".to_string(),
        rules,
    };

    system.iterate(3);
}
