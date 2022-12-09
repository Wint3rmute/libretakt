//! Lets the [DSP Engine](crate::engine) know what notes to play at what time.
//!
//! # Nomenclature
//! [Sequencer] stores all the notes to play, within a tree structure:
//! - [Sequencer] stores a list of [Tracks](Track)
//!   - [Track] stores a list of [Patterns](Pattern)
//!     - [Pattern] stores a list of [Steps](Step)
//!       - [Step] stores a list of optional parameter locks
//!
//! [Track] is played by a single voice of the [DSP Engine](crate::engine).
//! It is usually used to play a single *part* within a song. For example, one could say:
//! - "Track 1 plays the kick drum"
//! - "Track 2 plays the snares"
//! - "Track 3 plays the base line samples"
//! - "Track 4 plays the synth lead"
//!

extern crate serde;
extern crate serde_derive;

use crate::constants::{self, NUM_OF_VOICES};
use crate::engine::core::Voice;
use flume::{Receiver, Sender};
use log::{debug, error};
use serde::{Deserialize, Serialize};

use common::Parameter;
use common::SequencerMutation;
use common::NUM_OF_PARAMETERS;

use flume::bounded;

// 0.17.1

/// Allows lock-free synchronisation between multiple [Sequencer] instances,
/// shared across different threads.
///
/// Why not just use [Mutexes](std::sync::Mutex), you might be wandering.
/// [Here's a good explanation](https://timur.audio/using-locks-in-real-time-audio-processing-safely)
#[derive(Default)]
pub struct SynchronisationController {
    senders: Vec<Sender<SequencerMutation>>,
}

impl SynchronisationController {
    /// Returns a new rx channel, which you can pass to a [Sequencer] to keep it synchronised.
    pub fn register_new(&mut self) -> Receiver<SequencerMutation> {
        let (mutations_tx, mutations_rx) = bounded::<SequencerMutation>(64);
        self.senders.push(mutations_tx);

        mutations_rx
    }

    /// Applies a mutation to all registered [Sequencers](Sequencer).
    pub fn mutate(&mut self, mutation: SequencerMutation) {
        for sender in &self.senders {
            sender.send(mutation.clone()).unwrap();
        }
    }
}

pub type CurrentStepData = [usize; 8];

/// Main clock for all [Tracks](Track), triggers [Steps](Step) at the right time.
///
/// **Important:** this structure is not shared within threads and **should not be mutated directly**.
/// To synchronise mutations across different copies of the Sequencer, use the [SynchronisationController].
pub struct Sequencer {
    pub tracks: Vec<Track>,
    pub beats_per_minute: u8,
    pub time_counter: usize,
    pub mutations_queue: Receiver<SequencerMutation>,
    pub current_step_sender: Sender<CurrentStepData>,
}

impl Sequencer {
    pub fn new(
        mutations_queue: Receiver<SequencerMutation>,
        current_step_sender: Sender<CurrentStepData>,
        tracks: Vec<Track>,
    ) -> Self {
        Sequencer {
            tracks: tracks,
            beats_per_minute: 120,
            time_counter: 0,
            mutations_queue,
            current_step_sender,
        }
    }

    /// Lock-free synchronisation of [Sequencer]s between threads.
    /// Should be called as often as possible :)
    pub fn apply_mutations(&mut self) {
        while let Ok(mutation) = self.mutations_queue.try_recv() {
            debug!("Got mutation");
            match mutation {
                SequencerMutation::CreateStep(track, pattern, step) => {
                    self.tracks[track].patterns[pattern].steps[step] = Some(Step::default());
                }
                SequencerMutation::RemoveStep(track, pattern, step) => {
                    self.tracks[track].patterns[pattern].steps[step] = None;
                }
                SequencerMutation::SetParam(track, pattern, step, parameter, value) => {
                    if self.tracks[track].patterns[pattern].steps[step].is_none() {
                        self.tracks[track].patterns[pattern].steps[step] = Some(Step::default());
                    }

                    self.tracks[track].patterns[pattern].steps[step]
                        .as_mut()
                        .unwrap()
                        .parameters[parameter as usize] = Some(value);
                }
                SequencerMutation::RemoveParam(track, pattern, step, parameter) => {
                    if self.tracks[track].patterns[pattern].steps[step].is_none() {
                        self.tracks[track].patterns[pattern].steps[step] = Some(Step::default());
                    }

                    self.tracks[track].patterns[pattern].steps[step]
                        .as_mut()
                        .unwrap()
                        .parameters[parameter as usize] = None;
                }
                SequencerMutation::UpdateTrackParam(track, index, value) => {
                    self.tracks[track].default_parameters.parameters[index] = value;
                }
                SequencerMutation::SilenceTrack(track) => {
                    self.tracks[track].silenced = true;
                }
                SequencerMutation::UnSilenceTrack(track) => {
                    self.tracks[track].silenced = false;
                }
            }
        }
    }

    /// Should be called with the sound generation clock,
    /// allows for sample-perfect (aka pixel-perfect) sequence timing
    pub fn tick(&mut self, voices: &mut [Voice]) {
        self.time_counter += 1;

        // Dividing by 4 in the end to use eight-notes as default step length,
        // it will feel more intuitive to the user this way (trust me)
        if self.time_counter as f32
            >= 60.0 / self.beats_per_minute as f32 * constants::SAMPLE_RATE as f32 / 4.0
        {
            self.time_counter = 0;
            self.play_step(voices);
        }
    }

    fn play_step(&mut self, voices: &mut [Voice]) {
        let mut step_data: CurrentStepData = [0; 8];
        for (track_num, (track, voice)) in self.tracks.iter_mut().zip(voices.iter_mut()).enumerate()
        {
            if let Some(parameters) = track.next_step() {
                voice.play(parameters);
            }

            step_data[track_num] = track.current_step;
        }
        match self.current_step_sender.try_send(step_data) {
            Ok(_) => {}
            Err(_) => {
                error!("Unable to send step data, is the UI thread too slow?");
            }
        }
    }
}

/// Represents a single track within a song.
///
/// Each [Track](Track) has a default value for all [PlaybackParameters](PlaybackParameters),
/// but each [Step](Step) can override them using *parameter locks*.
#[derive(Serialize, Deserialize, Clone)]
pub struct Track {
    pub default_parameters: PlaybackParameters,
    pub patterns: Vec<Pattern>,
    pub current_pattern: usize,
    pub current_step: usize,
    pub silenced: bool,
}

/// Variation of a melody played within a [Track].
#[derive(Serialize, Deserialize, Clone)]
pub struct Pattern {
    pub steps: Vec<Option<Step>>,
}

impl Pattern {
    fn new() -> Self {
        Self {
            steps: vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
        }
    }
}

impl Track {
    pub fn new() -> Self {
        Track {
            default_parameters: PlaybackParameters::default(),
            patterns: vec![Pattern::new()],
            silenced: false,
            current_step: 0,
            current_pattern: 0,
        }
    }

    /// Advances a step forward in the track and returns an Step to play, if it exists
    /// Applies all parameter locks defined in the step to default [PlaybackParameters](Track::playback_parameters).
    fn next_step(&mut self) -> Option<PlaybackParameters> {
        self.current_step += 1;
        let pattern = &self.patterns[self.current_pattern];

        if self.current_step >= pattern.steps.len() {
            self.current_step = 0;
        }

        if self.silenced {
            return None;
        }

        if let Some(step) = &pattern.steps[self.current_step] {
            let playback_parameters = self.default_parameters.merge(step);
            // TODO: parameter locks
            Some(playback_parameters)
        } else {
            None
        }
    }
}

/// Defines how a sample should be played.
///
/// Used both as default playback parameters for a track
/// and as an input message for [Voice](crate::engine::Voice).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaybackParameters {
    #[allow(dead_code)] // TODO: remove after parameter locks are added
    pub parameters: [u8; NUM_OF_PARAMETERS],
}

impl Default for PlaybackParameters {
    fn default() -> Self {
        let mut parameters = [0u8; NUM_OF_PARAMETERS];
        parameters[Parameter::Note as usize] = 64u8;
        parameters[Parameter::PitchShift as usize] = 20u8;
        parameters[Parameter::Sample as usize] = 0u8;
        parameters[Parameter::FilterCutoff as usize] = 63u8;
        parameters[Parameter::NoteVelocity as usize] = 63u8;
        parameters[Parameter::ReverbEarlyMix as usize] = 63u8;
        parameters[Parameter::ReverbSize as usize] = 63u8;
        parameters[Parameter::ReverbDecay as usize] = 63u8;
        parameters[Parameter::SampleEnd as usize] = 63u8;

        PlaybackParameters { parameters }
    }
}

impl PlaybackParameters {
    fn merge(&self, step: &Step) -> Self {
        let mut result = self.clone();

        for index in 0..self.parameters.len() {
            if let Some(value) = step.parameters[index] {
                result.parameters[index] = value;
            }
        }

        result
    }
}

/// Represents a single step event, saved within a [Track](Track).
///
/// Contains possible overrides for playback parameters.
/// May also be reffered to as "*note on*" event in other samplers/sequencers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Step {
    #[allow(dead_code)] // TODO: remove after parameter locks are added
    pub parameters: [Option<u8>; NUM_OF_PARAMETERS],
}

impl Default for Step {
    fn default() -> Self {
        let parameters = [None; NUM_OF_PARAMETERS];
        Step { parameters }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_serialisation() {
        assert_eq!(Parameter::Note as u8, 0);
        assert_eq!(Parameter::Sample as u8, 2);
        // TODO: think of some cool way to convert those enum values to
        // binary messages efficiently, while reserving type safety
    }

    #[test]
    fn test_parameters_merge() {
        let parameters = PlaybackParameters::default();
        let mut step = Step::default();
        step.parameters[Parameter::Sample as usize] = Some(20u8);

        let merged = parameters.merge(&step);
        assert_eq!(merged.parameters[Parameter::Sample as usize], 20u8);
    }
}
