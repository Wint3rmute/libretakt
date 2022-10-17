//! Responsible for sample playback.
use rodio::Source;

use crate::sample_provider::{SampleData, SampleProvider};
use crate::sequencer::{Parameter, Sequencer};
use crate::{constants, sequencer::PlaybackParameters};
use std::sync::Arc;
use synfx_dsp::{DattorroReverb, DattorroReverbParams};

/// Top-level component of the DSP pipeline.
///
/// 1. Acts as a final [Source](rodio::Source) for the sampler.
/// 2. Owns all the [Voices](Voice) and takes care of sound generation.
/// 3. Acts as a mixer for all [Voices](Voice).
pub struct Engine {
    pub voices: Vec<Voice>,
    pub sequencer: Sequencer,
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
        self.sequencer.apply_mutations();
        self.sequencer.tick(&mut self.voices);

        Some(self.voices.iter_mut().map(|voice| voice.tick()).sum())
    }
}

pub struct ReverbParams {}

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
    pub playback_parameters: Option<PlaybackParameters>,
    pub play_position: f32,
    pub sample_played: usize,
    pub playback_speed: f32,
    pub reverb: DattorroReverb,
    pub reverb_params: ReverbParams,
}

impl DattorroReverbParams for ReverbParams {
    fn pre_delay_time_ms(&self) -> f64 {
        0.0
    }

    fn time_scale(&self) -> f64 {
        0.5
    }

    fn input_low_cutoff_hz(&self) -> f64 {
        22000.0
    }

    fn input_high_cutoff_hz(&self) -> f64 {
        0.0
    }

    fn reverb_high_cutoff_hz(&self) -> f64 {
        0.0
    }

    fn reverb_low_cutoff_hz(&self) -> f64 {
        0.0
    }

    fn mod_speed(&self) -> f64 {
        0.0
    }

    fn mod_depth(&self) -> f64 {
        0.0
    }

    fn mod_shape(&self) -> f64 {
        0.5
    }

    fn input_diffusion_mix(&self) -> f64 {
        1.0
    }

    fn diffusion(&self) -> f64 {
        0.5
    }

    fn decay(&self) -> f64 {
        0.6
    }
}

impl Voice {
    fn get_at_index(&self, sample: &SampleData, sample_position: f32) -> f32 {
        let left_sample = sample_position.floor();
        let right_sample = left_sample + 1.0;

        let distance_from_left_sample = sample_position - left_sample;
        let distance_from_right_sample = 1.0 - distance_from_left_sample;

        (sample.data[left_sample as usize] as f32 * (sample_position - left_sample))
            + (sample.data[right_sample as usize] as f32 * distance_from_right_sample)
    }

    pub fn play(&mut self, parameters: PlaybackParameters) {
        self.playback_parameters = Some(parameters);
        self.reset()
    }

    pub fn reset(&mut self) {
        self.play_position = 0.0;
    }

    pub fn new(provider: &Arc<SampleProvider>) -> Self {
        let mut reverb = DattorroReverb::new();
        reverb.set_sample_rate(constants::SAMPLE_RATE as f64);

        Voice {
            sample_provider: provider.clone(),
            play_position: 0.0,
            sample_played: 1,
            playback_speed: 1.0,
            reverb,
            reverb_params: ReverbParams {},
            playback_parameters: None,
        }
    }

    /// TLDR linear interpolation for sample playback,
    /// allowing for speeding up and slowing down samples:
    ///
    /// Each sound has a "playback position", from 0.0 to <num_of_samples>.
    /// When a next sample is requested, it is calculated as follows:
    /// - Find the 2 samples closest to the playback position
    /// - Return a weighted average
    ///
    /// Example:
    /// Position = 112.2
    /// Total number of points in the sample: 128
    ///
    /// 128 * 0.23 = 29.44
    /// Distance from:
    ///     - sample 112 => 0.2
    ///     - sample 113 => 0.8
    ///
    /// Result: avg(
    ///     Sample 112 * 0.2
    ///     Sample 113 * 0.8
    /// )
    fn tick(&mut self) -> f32 {
        if let Some(parameters) = &self.playback_parameters {
            let sample = &self.sample_provider.samples
                [parameters.parameters[Parameter::Sample as usize] as usize];

            if (self.play_position + 1.0) >= sample.data.len() as f32 {
                0.0
            } else {
                let result = self.get_at_index(sample, self.play_position);
                self.play_position += self.playback_speed;

                result
                // + self
                //     .reverb
                //     .process(&mut self.reverb_params, result as f64, result as f64)
                //     .0 as f32
            }
        } else {
            0.0
        }
    }
}
