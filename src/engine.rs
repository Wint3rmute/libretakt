//! DSP engine module the contains code responsible for sample playback.

pub struct Engine {}

/// An audible voice, playing a single sample at a time.
/// Receives events from the [Track](crate::sequencer::Track).
///
/// Applies various playback modifiers and effects to the played sound,
/// depending on the [Step](crate::sequencer::Step) data received and default
/// [Track](crate::sequencer::Track) settings.
pub struct Voice {}
