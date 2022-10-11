//! DSP engine module the contains code responsible for sample playback.
use rodio::Source;

use crate::constants;
use crate::sample_provider::SampleProvider;
use crate::sequencer::Sequencer;
use std::sync::{Arc, RwLock};

/// Top-level component of the DSP pipeline.
///
/// 1. Acts as a final [Source](rodio::Source) for the sampler.
/// 2. Owns all the [Voices](Voice) and takes care of sound generation.
/// 3. Acts as a mixer for all [Voices](Voice).
pub struct Engine {
    pub voices: Vec<Voice>,
    pub sequencer: Arc<RwLock<Sequencer>>,
}

impl Source for Engine {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        constants::SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for Engine {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.sequencer.write().unwrap().tick(&mut self.voices);

        Some(self.voices.iter_mut().map(|voice| voice.tick()).sum())
    }
}

/// An audible voice, playing a single sample at a time.
/// Receives events from the [Track](crate::sequencer::Track).
///
/// Applies various playback modifiers and effects to the played sound,
/// depending on the [Step](crate::sequencer::Step) data received and default
/// [Track](crate::sequencer::Track) settings.
pub struct Voice {
    // sample_provider: Arc<RwLock<SampleProvider>>,
    // pub sample_provider: &'a SampleProvider,
    pub sample_provider: Arc<SampleProvider>,
    pub play_position: usize,
}

impl Voice {
    pub fn reset(&mut self) {
        self.play_position = 0;
    }

    pub fn new(provider: &Arc<SampleProvider>) -> Self {
        Voice {
            sample_provider: provider.clone(),
            play_position: 0,
        }
    }
    fn tick(&mut self) -> f32 {
        let sample = &self.sample_provider.samples[0];

        if self.play_position >= sample.data.len() {
            0.0 // Silence if end of sample reached
        } else {
            let result = sample.data[self.play_position];
            self.play_position += 1;
            result
        }
    }
}
