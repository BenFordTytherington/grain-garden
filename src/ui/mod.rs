pub mod delay_ui;
pub mod grain_ui;
pub mod plant_ui;

pub use delay_ui::DelayUi;
use egui::Response;
pub use grain_ui::GranularUi;
pub use plant_ui::LSystemUi;
use std::sync::mpsc::Sender;

pub fn send_params<T: Send>(sender: &Sender<T>, params: T) {
    sender
        .send(params)
        .unwrap_or_else(|e| panic!("Failed to send params with error: {}", e))
}

fn call_on_change(f: impl FnOnce(), responses: &[Response]) {
    if responses.iter().any(|r| r.changed()) {
        f()
    }
}
