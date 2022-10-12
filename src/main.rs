use libretakt::sequencer::Parameters;
use macroquad::prelude::*;

use macroquad::telemetry::frame;
use macroquad::ui::{hash, root_ui};
use rodio::{OutputStream, Sink};
use std::sync::{Arc, RwLock};

pub mod constants;
pub mod engine;
pub mod sample_provider;
pub mod sequencer;

use engine::{Engine, Voice};

#[macroquad::main("LibreTakt")]
async fn main() {
    let provider = Arc::new(sample_provider::SampleProvider::default());

    let sequencer = Arc::new(RwLock::new(sequencer::Sequencer::new()));
    let voice = Voice::new(&provider);
    let engine = Engine {
        sequencer: sequencer.clone(),
        voices: vec![voice],
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(engine);
    sink.play();

    let mut sample = 0.0;

    loop {
        clear_background(BLACK);

        {
            let mut sequencer = sequencer.write().unwrap();
            sequencer.tracks[0].default_parameters.parameters[Parameters::Sample as usize] =
                sample as u8;
            let current_pattern = &mut sequencer.tracks[0].patterns[0]; // Hardcoded
            let num_of_steps = current_pattern.steps.len();

            for i in 0..num_of_steps {
                root_ui().slider(hash!(), "[-10 .. 10]", 0f32..10f32, &mut sample);
                if root_ui().button(
                    None,
                    if current_pattern.steps[i].is_some() {
                        "X"
                    } else {
                        " "
                    },
                ) {
                    if current_pattern.steps[i].is_some() {
                        current_pattern.steps[i] = None;
                    } else {
                        current_pattern.steps[i] = Some(sequencer::Step::default());
                    }
                }
            }
        }
        next_frame().await;
    }
}
