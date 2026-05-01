/// Local sequencer UI state — not shared over the network.
/// The authoritative track data lives in `ApplicationState::sequencer` and is
/// kept in sync with the server via `ServerMessage` updates.
#[derive(Default)]
pub struct LocalSequencerState {
    pub current_step: usize,
}
