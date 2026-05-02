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
use arbitrary::Arbitrary;
use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use common::MutationWithSource;
use common::Parameter;
use common::SequencerMutation;
use common::NUM_OF_PARAMETERS;

pub type CurrentStepData = [usize; 8];

/// Main clock for all [Tracks](Track), triggers [Steps](Step) at the right time.
///
/// **Important:** this structure is not shared within threads and **should not be mutated directly**.
/// To synchronise mutations across different copies of the Sequencer, use the [SynchronisationController].
pub struct Sequencer {
    pub tracks: Vec<Track>,
    pub beats_per_minute: u8,
    pub time_counter: usize,
    pub mutations_queue: Receiver<MutationWithSource>,
    pub current_step_sender: Sender<CurrentStepData>,
    pub playing: bool,
}

impl Sequencer {
    pub fn new(
        mutations_queue: Receiver<MutationWithSource>,
        current_step_sender: Sender<CurrentStepData>,
        tracks: Vec<Track>,
    ) -> Self {
        Sequencer {
            tracks: tracks,
            beats_per_minute: 120,
            time_counter: 0,
            mutations_queue,
            current_step_sender,
            playing: false,
        }
    }

    fn start_playback(&mut self) {
        self.playing = true;
    }

    fn stop_playback(&mut self) {
        for track in self.tracks.iter_mut() {
            track.current_step = 0;
        }

        self.playing = false;
        self.time_counter = 0;
    }

    /// Lock-free synchronisation of [Sequencer]s between threads.
    /// Should be called as often as possible :)
    pub fn apply_mutations(&mut self) {
        while let Ok(mutation) = self.mutations_queue.try_recv() {
            debug!("Got mutation");

            let mutation = match mutation {
                MutationWithSource::Local(mutation) => mutation,
                MutationWithSource::Remote(mutation) => mutation,
            };

            match mutation {
                SequencerMutation::CreateStep(track, pattern, step) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "CreateStep: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    let Some(p) = t.patterns.get_mut(pattern) else {
                        tracing::warn!(
                            "CreateStep: pattern {pattern} out of range (len={})",
                            t.patterns.len()
                        );
                        continue;
                    };
                    let Some(s) = p.steps.get_mut(step) else {
                        tracing::warn!(
                            "CreateStep: step {step} out of range (len={})",
                            p.steps.len()
                        );
                        continue;
                    };
                    *s = Some(Step::default());
                }
                SequencerMutation::RemoveStep(track, pattern, step) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "RemoveStep: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    let Some(p) = t.patterns.get_mut(pattern) else {
                        tracing::warn!(
                            "RemoveStep: pattern {pattern} out of range (len={})",
                            t.patterns.len()
                        );
                        continue;
                    };
                    let Some(s) = p.steps.get_mut(step) else {
                        tracing::warn!(
                            "RemoveStep: step {step} out of range (len={})",
                            p.steps.len()
                        );
                        continue;
                    };
                    *s = None;
                }
                SequencerMutation::SetParam(track, pattern, step, parameter, value) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "SetParam: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    let Some(p) = t.patterns.get_mut(pattern) else {
                        tracing::warn!(
                            "SetParam: pattern {pattern} out of range (len={})",
                            t.patterns.len()
                        );
                        continue;
                    };
                    let Some(s) = p.steps.get_mut(step) else {
                        tracing::warn!(
                            "SetParam: step {step} out of range (len={})",
                            p.steps.len()
                        );
                        continue;
                    };
                    if s.is_none() {
                        *s = Some(Step::default());
                    }
                    s.as_mut().unwrap().parameters[parameter as usize] = Some(value);
                }
                SequencerMutation::RemoveParam(track, pattern, step, parameter) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "RemoveParam: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    let Some(p) = t.patterns.get_mut(pattern) else {
                        tracing::warn!(
                            "RemoveParam: pattern {pattern} out of range (len={})",
                            t.patterns.len()
                        );
                        continue;
                    };
                    let Some(s) = p.steps.get_mut(step) else {
                        tracing::warn!(
                            "RemoveParam: step {step} out of range (len={})",
                            p.steps.len()
                        );
                        continue;
                    };
                    if s.is_none() {
                        *s = Some(Step::default());
                    }
                    s.as_mut().unwrap().parameters[parameter as usize] = None;
                }
                SequencerMutation::UpdateTrackParam(track, index, value) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "UpdateTrackParam: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    let Some(p) = t.default_parameters.parameters.get_mut(index) else {
                        tracing::warn!(
                            "UpdateTrackParam: param index {index} out of range (len={})",
                            t.default_parameters.parameters.len()
                        );
                        continue;
                    };
                    *p = value;
                }
                SequencerMutation::SilenceTrack(track) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "SilenceTrack: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    t.silenced = true;
                }
                SequencerMutation::UnSilenceTrack(track) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "UnSilenceTrack: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    t.silenced = false;
                }
                SequencerMutation::SelectPattern(track, pattern) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "SelectPattern: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    if pattern >= t.patterns.len() {
                        tracing::warn!(
                            "SelectPattern: pattern {pattern} out of range (len={})",
                            t.patterns.len()
                        );
                        continue;
                    }
                    t.current_pattern = pattern;
                }
                SequencerMutation::StopPlayback => self.stop_playback(),
                SequencerMutation::StartPlayback => self.start_playback(),
                SequencerMutation::SetTrackLength(track, new_length) => {
                    let Some(t) = self.tracks.get_mut(track) else {
                        tracing::warn!(
                            "SetTrackLength: track {track} out of range (len={})",
                            self.tracks.len()
                        );
                        continue;
                    };
                    t.set_length(new_length);
                }
            }
        }
    }

    /// Should be called with the sound generation clock,
    /// allows for sample-perfect (aka pixel-perfect) sequence timing
    pub fn tick(&mut self, voices: &mut [Voice]) {
        if !self.playing {
            return;
        }
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
            patterns: vec![
                Pattern::new(),
                Pattern::new(),
                Pattern::new(),
                Pattern::new(),
            ],
            silenced: false,
            current_step: 0,
            current_pattern: 0,
        }
    }

    /// Advances a step forward in the track and returns an Step to play, if it exists
    /// Applies all parameter locks defined in the step to default [PlaybackParameters](Track::playback_parameters).
    fn next_step(&mut self) -> Option<PlaybackParameters> {
        self.current_step += 1;
        let pattern = self.patterns.get(self.current_pattern)?;

        if self.current_step >= pattern.steps.len() {
            self.current_step = 0;
        }

        if self.silenced {
            return None;
        }

        // Use get() so an empty step list (e.g. after set_length(0)) never panics.
        // TODO: parameter locks
        pattern
            .steps
            .get(self.current_step)
            .and_then(|s| s.as_ref())
            .map(|step| self.default_parameters.merge(step))
    }

    fn set_length(&mut self, new_length: usize) {
        if new_length > self.patterns[0].steps.len() {
            for _ in 0..new_length - self.patterns[0].steps.len() {
                for pattern in self.patterns.iter_mut() {
                    pattern.steps.push(None);
                }
            }
        } else {
            for _ in 0..self.patterns[0].steps.len() - new_length {
                for pattern in self.patterns.iter_mut() {
                    pattern.steps.pop();
                }
            }
        }
    }
}

/// Defines how a sample should be played.
///
/// Used both as default playback parameters for a track
/// and as an input message for [Voice](crate::engine::Voice).
#[derive(Serialize, Deserialize, Debug, Clone, Arbitrary)]
pub struct PlaybackParameters {
    #[allow(dead_code)] // TODO: remove after parameter locks are added
    pub parameters: [u8; NUM_OF_PARAMETERS],
}

impl Default for PlaybackParameters {
    fn default() -> Self {
        let mut parameters = [0u8; NUM_OF_PARAMETERS];
        parameters[Parameter::Note as usize] = 32u8;
        parameters[Parameter::PitchShift as usize] = 16u8;
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
