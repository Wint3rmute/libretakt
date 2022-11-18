//! Responsible for sample playback.
use rodio::Source;

use crate::ladder_filter;
use crate::mverb;
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
    pub playback_parameters: PlaybackParameters,
    pub play_position: f32,
    pub sample_played: usize,
    pub playback_speed: f32,
    // pub reverb: reverb::DattorroReverbF32,
    pub mverb: mverb::MVerb,
    pub delay: mverb::AllPass<44100>,

    pub delay_send: f32,
    pub reverb_send: f32,

    pub ladder_filter: ladder_filter::LadderFilter,

    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub b3: f32,
    pub filter_delay: [f32; 4],
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
        self.playback_speed = parameters.parameters[Parameter::PitchShift as usize] as f32 / 20.0;
        // self.playback_speed = 1.0;

        // self.reverb_params
        //     .fill(parameters.parameters[Parameter::PitchShift as usize as usize] as f32 / 64.0);
        self.playback_parameters = parameters;
        let ref parameters = self.playback_parameters.parameters;

        self.mverb.mix = parameters[Parameter::ReverbParamIdkWhatYet1 as usize] as f32 / 64.0;

        self.mverb.decay = parameters[Parameter::ReverbParamIdkWhatYet2 as usize] as f32 / 64.0;

        self.delay_send = parameters[Parameter::DelaySend as usize] as f32 / 64.0;
        self.reverb_send = parameters[Parameter::ReverbSend as usize] as f32 / 64.0;

        self.delay
            .set_length(parameters[Parameter::DelayTime as usize] as usize * 1000);

        self.ladder_filter.params.set_cutoff(
            self.playback_parameters.parameters[Parameter::FilterCutoff as usize] as f32 / 64.0,
        );

        self.ladder_filter.params.set_cutoff(
            self.playback_parameters.parameters[Parameter::FilterCutoff as usize] as f32 / 64.0,
        );
        self.ladder_filter.params.res =
            self.playback_parameters.parameters[Parameter::FilterResonance as usize] as f32 / 64.0
                * 2.0;
        // * 0.4;

        self.reset();
    }

    pub fn reset(&mut self) {
        self.play_position = 0.0;
    }

    pub fn new(provider: &Arc<SampleProvider>) -> Self {
        // let mut reverb = reverb::DattorroReverbF32::new();
        // reverb.set_sample_rate(constants::SAMPLE_RATE as f32);

        let mut parameters = PlaybackParameters::default();
        parameters.parameters[Parameter::Sample as usize] = 126;

        let mut ladder_filter = ladder_filter::LadderFilter::new();
        ladder_filter.params.sample_rate = constants::SAMPLE_RATE as f32;
        // ladder_filter.params.set_cutoff(20000.0);

        Voice {
            sample_provider: provider.clone(),
            play_position: 0.0,
            sample_played: 1,
            playback_speed: 1.0,
            // reverb,
            mverb: mverb::MVerb::default(),
            delay: mverb::AllPass::default(),
            playback_parameters: parameters,
            ladder_filter,

            reverb_send: 0.0,
            delay_send: 0.0,

            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            filter_delay: [0.0; 4],
        }
    }

    // ladder_filter.params.s;

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

    fn tick(&mut self) -> f32 {
        let sample_raw = self.get_next_raw_sample_and_progress();

        let freq =
            self.playback_parameters.parameters[Parameter::FilterCutoff as usize] as f32 * 200.0;
        let resonance =
            self.playback_parameters.parameters[Parameter::FilterResonance as usize] as f32 / 64.0;
        // let (sample_filtered, _band, _high) = process_simper_svf(
        //     sample_raw,
        //     freq,
        //     resonance,
        //     1.0 / constants::SAMPLE_RATE as f32,
        //     &mut self.b0,
        //     &mut self.b1,
        // );

        let sample_filtered = self.ladder_filter.process(sample_raw);

        // if sample_filtered != 0.0 {
        //     println!("{sample_filtered}");
        // }

        // let sample_filtered = sample_raw;

        let delay_effect = self.delay.operator(sample_filtered * self.delay_send);

        let reverb_in = (sample_filtered + delay_effect) * self.reverb_send;
        let reverb_effect = self.mverb.process((reverb_in, reverb_in));

        sample_filtered + delay_effect + reverb_effect.0
        //
        // let reverb_effect = self.mverb.process((sample_filtered, sample_filtered));

        // reverb_effect.0
    }
}
