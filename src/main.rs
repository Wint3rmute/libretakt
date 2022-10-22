use libretakt::engine::{Engine, Voice};
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::{Parameter, Sequencer, SequencerMutation, SynchronisationController};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use rodio::{OutputStream, Sink};
use std::sync::Arc;

#[macroquad::main("LibreTakt")]
async fn main() {
    let provider = Arc::new(SampleProvider::default());

    let mut synchronisation_controller = SynchronisationController::default();

    let voice = Voice::new(&provider);
    let engine = Engine {
        sequencer: Sequencer::new(synchronisation_controller.register_new()),
        voices: vec![voice],
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(engine);
    sink.play();

    let sequencer = Sequencer::new(synchronisation_controller.register_new());

    ui_main(sequencer, synchronisation_controller).await;
}

async fn ui_main(
    mut sequencer: Sequencer,
    mut synchronisation_controller: SynchronisationController,
) {
    let mut sample = 0.0;

    loop {
        {
            // Alias the name to make the sequencer immutable for the drawing code.
            let sequencer = &sequencer; // Do not change to mutable!
            clear_background(BLACK);

            let current_pattern = &sequencer.tracks[0].patterns[0]; // Hardcoded for now
            let num_of_steps = current_pattern.steps.len();

            let fps = get_fps();
            root_ui().button(None, format!("{fps}"));

            synchronisation_controller.mutate(SequencerMutation::SetParam(
                0,
                0,
                0,
                Parameter::Sample,
                sample as u8,
            ));

            widgets::Window::new(
                hash!(),
                vec2(0.0, 0.0),
                vec2(screen_width() / 2.0, screen_height()),
            )
            .label("whatever")
            .ui(&mut *root_ui(), |ui| {
                for i in 0..num_of_steps {
                    ui.slider(hash!(), "[-10 .. 10]", 0f32..10f32, &mut sample);
                    if ui.button(
                        None,
                        if current_pattern.steps[i].is_some() {
                            "X"
                        } else {
                            " "
                        },
                    ) {
                        if current_pattern.steps[i].is_some() {
                            synchronisation_controller
                                .mutate(SequencerMutation::RemoveStep(0, 0, i));
                        } else {
                            synchronisation_controller
                                .mutate(SequencerMutation::CreateStep(0, 0, i));
                        }
                    }
                }
            });
        }
        sequencer.apply_mutations();
        next_frame().await;
    }
}
