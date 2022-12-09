extern crate serde;
extern crate serde_derive;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter; // 0.17.1

pub type TrackNum = usize;
pub type PatternNum = usize;
pub type StepNum = usize;
pub type ParamValue = u8;
pub type ParamNum = usize;

/// Represents a single change applied to the [Sequencer] structure
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequencerMutation {
    CreateStep(TrackNum, PatternNum, StepNum),
    RemoveStep(TrackNum, PatternNum, StepNum),
    SetParam(TrackNum, PatternNum, StepNum, Parameter, ParamValue),
    RemoveParam(TrackNum, PatternNum, StepNum, Parameter),
    UpdateTrackParam(TrackNum, ParamNum, ParamValue),
    SilenceTrack(TrackNum),
    UnSilenceTrack(TrackNum),
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromPrimitive, EnumIter)]
#[repr(u8)]
pub enum Parameter {
    // Page 1: playback
    Note = 0,
    PitchShift,
    Sample,   // Only changes in integer values
    PlayMode, // Only changes in integer values
    NoteLength,
    NoteVelocity,
    SampleStart,
    SampleEnd,

    // Page 2: filtering
    FilterCutoff,
    FilterResonance,
    FilterAttack,
    FilterDecay,
    FilterSustain,
    FilterRelease,
    FilterType, // Only changes in integer values
    FilterEnvelope,

    // Page 3: effects
    ReverbSize,
    ReverbSend,
    ReverbDecay,
    ReverbEarlyMix,

    DelayTime,
    DelayFeedback,
    DelaySend,
    DelayParamIdkWhatYet1,
    // Remember: if adding new values to this enum, set the last value in NUM_OF_PARAMETERS below
}
pub const NUM_OF_PARAMETERS: usize = Parameter::DelayParamIdkWhatYet1 as usize + 1;

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{:?}", self)
    }
}
