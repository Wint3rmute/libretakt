//! Responsible for sample playback.
use rodio::Source;

use crate::mverb;
use crate::reverb;
use crate::sample_provider::{SampleData, SampleProvider};
use crate::sequencer::{Parameter, Sequencer};
use crate::{constants, sequencer::PlaybackParameters};
use std::sync::Arc;
use synfx_dsp::*;

/// Top-level component of the DSP pipeline.
///
/// 1. Acts as a final [Source](rodio::Source) for the sampler.
/// 2. Owns all the [Voices](Voice) and takes care of sound generation.
/// 3. Acts as a mixer for all [Voices](Voice).
pub struct Engine<'a> {
    pub voices: Vec<Voice<'a>>,
    pub sequencer: Sequencer,
}

impl<'a> Source for Engine<'a> {
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

impl<'a> Iterator for Engine<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.sequencer.apply_mutations();
        self.sequencer.tick(&mut self.voices);

        Some(self.voices.iter_mut().map(|voice| voice.tick()).sum())
    }
}

/// An audible voice, playing a single sample at a time.
/// Receives events from the [Track](crate::sequencer::Track).
///
/// Applies various playback modifiers and effects to the played sound,
/// depending on the [Step](crate::sequencer::Step) data received and default
/// [Track](crate::sequencer::Track) settings.
pub struct Voice<'a> {
    // sample_provider: Arc<RwLock<SampleProvider>>,
    // pub sample_provider: &'a SampleProvider,
    pub sample_provider: Arc<SampleProvider>,
    pub playback_parameters: PlaybackParameters,
    pub play_position: f32,
    pub sample_played: usize,
    pub playback_speed: f32,
    // pub reverb: reverb::DattorroReverbF32,
    pub mverb: mverb::MVerb<'a>,
    pub reverb_params: ReverbParams,

    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub b3: f32,
    pub filter_delay: [f32; 4],
}

pub struct ReverbParams {
    time_scale: f32,
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self { time_scale: 0.0 }
    }
}

impl ReverbParams {
    fn fill(&mut self, time_scale: f32) {
        self.time_scale = time_scale as f32;
    }
}

impl reverb::DattorroReverbParamsF32 for ReverbParams {
    fn pre_delay_time_ms(&self) -> f32 {
        0.0
    }

    fn time_scale(&self) -> f32 {
        self.time_scale
    }

    fn input_low_cutoff_hz(&self) -> f32 {
        22000.0
    }

    fn input_high_cutoff_hz(&self) -> f32 {
        0.0
    }

    fn reverb_high_cutoff_hz(&self) -> f32 {
        0.0
    }

    fn reverb_low_cutoff_hz(&self) -> f32 {
        0.0
    }

    fn mod_speed(&self) -> f32 {
        0.0
    }

    fn mod_depth(&self) -> f32 {
        0.0
    }

    fn mod_shape(&self) -> f32 {
        0.0
    }

    fn input_diffusion_mix(&self) -> f32 {
        1.0
    }

    fn diffusion(&self) -> f32 {
        0.0
    }

    fn decay(&self) -> f32 {
        self.time_scale
    }
}

impl<'a> Voice<'a> {
    fn get_at_index(&self, sample: &SampleData, sample_position: f32) -> f32 {
        let left_sample = sample_position.floor();
        let right_sample = left_sample + 1.0;

        let distance_from_left_sample = sample_position - left_sample;
        let distance_from_right_sample = 1.0 - distance_from_left_sample;

        (sample.data[left_sample as usize] as f32 * (sample_position - left_sample))
            + (sample.data[right_sample as usize] as f32 * distance_from_right_sample)
    }

    pub fn play(&mut self, parameters: PlaybackParameters) {
        // self.playback_speed = parameters.parameters[Parameter::PitchShift as usize] as f32 / 20.0;
        self.playback_speed = 1.0;

        self.reverb_params
            .fill(parameters.parameters[Parameter::PitchShift as usize as usize] as f32 / 64.0);
        self.playback_parameters = parameters;
        self.reset()
    }

    pub fn reset(&mut self) {
        self.play_position = 0.0;
    }

    pub fn new(provider: &Arc<SampleProvider>) -> Self {
        // let mut reverb = reverb::DattorroReverbF32::new();
        // reverb.set_sample_rate(constants::SAMPLE_RATE as f32);

        let mut parameters = PlaybackParameters::default();
        parameters.parameters[Parameter::Sample as usize] = 126;

        Voice {
            sample_provider: provider.clone(),
            play_position: 0.0,
            sample_played: 1,
            playback_speed: 1.0,
            // reverb,
            mverb: mverb::MVerb::default(),
            reverb_params: ReverbParams::default(),
            playback_parameters: parameters,

            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            filter_delay: [0.0; 4],
        }
    }

    fn get_next_raw_sample_and_progress(&mut self) -> f32 {
        if self.playback_parameters.parameters[Parameter::Sample as usize] as usize
            >= self.sample_provider.samples.len()
        {
            return 0.0;
        }
        let sample = &self.sample_provider.samples
            [self.playback_parameters.parameters[Parameter::Sample as usize] as usize];

        if (self.play_position + 1.0) >= sample.data.len() as f32 {
            0.0
        } else {
            let result = self.get_at_index(sample, self.play_position);
            self.play_position += self.playback_speed;
            result
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
        let result = self.get_next_raw_sample_and_progress();

        let freq =
            self.playback_parameters.parameters[Parameter::FilterCutoff as usize] as f32 * 200.0;
        let resonance =
            self.playback_parameters.parameters[Parameter::FilterResonance as usize] as f32 / 64.0;

        let (result, _band, _high) = process_simper_svf(
            result,
            freq,
            resonance,
            1.0 / constants::SAMPLE_RATE as f32,
            &mut self.b0,
            &mut self.b1,
        );

        // result

        // low
        // let (reverb_result, _) = self.reverb.process(&mut self.reverb_params, result, result);
        let reverb_result = self.mverb.process((result, result));

        reverb_result.0
    }
}
