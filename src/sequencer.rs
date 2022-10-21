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

use crate::constants;
use crate::engine::Voice;
use flume::{Receiver, Sender};
use serde::{Deserialize, Serialize};

use flume::bounded;

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

/// Represents a single change applied to the [Sequencer] structure
#[derive(Clone)]
pub enum SequencerMutation {
    CreateStep(usize, usize, usize),
    RemoveStep(usize, usize, usize),
    SetParam(usize, usize, usize, Parameter, u8),
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
    ) -> Self {
        Sequencer {
            tracks: vec![Track::new()],
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
            println!("Got mutation");
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
            >= 60.0 / self.beats_per_minute as f32 * constants::SAMPLE_RATE as f32 / 8.0
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
                println!("Unable to send step data, is the UI thread too slow?");
            }
        }
    }
}

/// Represents a single track within a song.
///
/// Each [Track](Track) has a default value for all [PlaybackParameters](PlaybackParameters),
/// but each [Step](Step) can override them using *parameter locks*.
#[derive(Serialize, Deserialize)]
pub struct Track {
    pub default_parameters: PlaybackParameters,
    pub patterns: Vec<Pattern>,
    pub current_pattern: usize,
    pub current_step: usize,
}

/// Variation of a melody played within a [Track].
#[derive(Serialize, Deserialize)]
pub struct Pattern {
    pub steps: Vec<Option<Step>>,
}

impl Pattern {
    fn new() -> Self {
        Self {
            steps: vec![
                Some(Step::default()),
                None,
                Some(Step::default()),
                None,
                None,
                None,
                None,
                None,
                Some(Step::default()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        }
    }
}

impl Track {
    fn new() -> Self {
        Track {
            default_parameters: PlaybackParameters::default(),
            patterns: vec![Pattern::new()],
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

        if let Some(step) = &pattern.steps[self.current_step] {
            let playback_parameters = self.default_parameters.merge(step);
            // TODO: parameter locks
            Some(playback_parameters)
        } else {
            None
        }
    }
}

/// Playback parameters, which are set by default for the entire track and can be overriden by *parameter locks*
///
/// All parameters are of [u8](u8) type, hence the allowed value of each parameter is between 1 and 127.
/// Below you can find a list of all possible parameters, which allow users to tweak the sound to their hearts' content:
/// ## Page 1: trigger
///
/// 1. Note - refer to <https://www.inspiredacoustics.com/en/MIDI_note_numbers_and_center_frequencies> for note-frequency mapping
///     - Center frequency of all samples is C4
///     - The link above contains an equation for calculating the desired frequency
/// 2. Note length:
///     - 0 - 0.0s
///     - 127 - 20s
///     - TLDR Just multiply the parameter value by 0.158 to get note length in seconds
/// 3. Note velocity:
///     - 0.0 all samples will be multiplied by 0.0, nothing will be heard
///     - 1.1 all samples will be multiplied by 1.0, sample will be played back at full volume
/// 4. Pitch shift - Allows for pitching samples up and down by fraction-of-a-note values
///     - value of 0 means that the sample is pitched down 1 octave (frequency is divided by 2)
///     - value of 127 means that the sample is pitched up by 1 octave (frequency is multiplied by 2)
///     - value of 64 means that the sample is played without any pitch shift
///     - TLDR in Python you would write `shift = lambda x: ((x-63.5)/63.5)+1`
/// 5. Play mode:
///     - 00-31  Forward
///     - 32-63  Reverse
///     - 64-95  Reverse loop
///     - 96-127 Forward loop
/// 6. Sample start:
///     - value/127.0 = procentowo od jakiego miejsca zacząć odtwarzanie idk jak to napisać po angielsku xd
///     - note: if play mode is set to reverse loop, the sample will 'wrap around' this point
/// 7. Sample end:
///     - value/127.0 - same as above, except instead of a starting point, this is an ending point
///     - note: if play mode is set to forward loop, the sample will 'wrap around' this point
/// 8. Sample select:
///     - Select the sample to use, there will be 127 available, no math needed here :)
///
/// ## Page 2: filter (all values 1-127 here)
///
/// 1. Filter attack
/// 2. Filter decay
/// 3. Filter sustain
/// 4. Filter release
/// 5. Filter cutoff
/// 6. Filter resonance
/// 7. Filter envelope intensity
/// 8. ???? (todo)
///
/// ## Page 3: Processing
///
/// 1. Sample attack
/// 2. Sample decay
/// 3. Sample release
/// 4. Delay send
/// 5. Reverb send
/// 6. Pan
/// 7. Reverb dry/wet
/// 8. Delay dry/wet
#[derive(Clone)]
#[repr(u8)]
pub enum Parameter {
    Note = 0,
    PitchShift,
    Sample, // Remember: if adding new values to this enum, set the last value in NUM_OF_PARAMETERS below
}
pub const NUM_OF_PARAMETERS: usize = Parameter::Sample as usize + 1;

/// Represents a single step event, saved within a [Track](Track).
///
/// Contains possible overrides for playback parameters.
/// May also be reffered to as "*note on*" event in other samplers/sequencers.
#[derive(Serialize, Deserialize, Debug)]
pub struct Step {
    #[allow(dead_code)] // TODO: remove after parameter locks are added
    parameters: [Option<u8>; NUM_OF_PARAMETERS],
}

impl Default for Step {
    fn default() -> Self {
        let parameters = [None; NUM_OF_PARAMETERS];
        Step { parameters }
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
        parameters[Parameter::PitchShift as usize] = 64u8;
        parameters[Parameter::Sample as usize] = 0u8;

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
