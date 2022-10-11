use macroquad::prelude::*;

use macroquad::ui::root_ui;
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

    loop {
        clear_background(BLACK);

        {
            let mut sequencer = sequencer.write().unwrap();
            let num_of_steps = sequencer.tracks[0].steps.len();

            for i in 0..num_of_steps {
                if root_ui().button(
                    None,
                    if sequencer.tracks[0].steps[i].is_some() {
                        "X"
                    } else {
                        " "
                    },
                ) {
                    if sequencer.tracks[0].steps[i].is_some() {
                        sequencer.tracks[0].steps[i] = None;
                    } else {
                        sequencer.tracks[0].steps[i] = Some(sequencer::Step::default());
                    }
                }
            }
        }
        next_frame().await;
    }
}
