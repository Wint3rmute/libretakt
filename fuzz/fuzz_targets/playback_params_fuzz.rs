#![no_main]

use once_cell::sync::Lazy;
use std::sync::Arc;

use libfuzzer_sys::fuzz_target;
use libretakt::constants;
use libretakt::engine::Voice;
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::PlaybackParameters;

static SAMPLE_PROVIDER: Lazy<Arc<SampleProvider>> =
    Lazy::new(|| Arc::new(SampleProvider::default()));

fuzz_target!(|parameters: PlaybackParameters| {
    let mut voice = Voice::new(&SAMPLE_PROVIDER);

    voice.play(parameters.clone());

    for _ in 1..constants::SAMPLE_RATE * 5 {
        let _result = voice.tick();
    }

    voice.play(PlaybackParameters::default());

    for _ in 1..constants::SAMPLE_RATE * 5 {
        if voice.tick() != 0.0 {
            return;
        }
    }

    panic!("No sound generated with default playback params");
});
