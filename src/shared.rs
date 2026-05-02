//! Types shared between the native server and the WASM client.
//!
//! This module is compiled for **both** targets — no `cfg` gates here.
//! All types must be `serde`-serializable so they can travel over the WebSocket as JSON.

use serde::{Deserialize, Serialize};

/// Identifies a connected client. Assigned by the server on connect.
pub type ClientId = u64;

/// State of a single sequencer track.
///
/// Lock ownership lives inside the track state so that a [`ServerMessage::TrackUpdate`]
/// is always self-contained: it carries both step data and current lock status.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TrackState {
    pub steps: Vec<bool>,
    /// `None` means the track is free to claim; `Some(id)` means it is locked by that client.
    pub locked_by: Option<ClientId>,
}

impl TrackState {
    #[must_use]
    pub fn new(num_steps: usize) -> Self {
        Self {
            steps: vec![false; num_steps],
            locked_by: None,
        }
    }
}

/// The canonical sequencer state. The authoritative copy lives on the server.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SequencerState {
    pub tracks: Vec<TrackState>,
}

impl SequencerState {
    #[must_use]
    pub fn new(num_tracks: usize, steps_per_track: usize) -> Self {
        Self {
            tracks: std::iter::repeat_with(|| TrackState::new(steps_per_track))
                .take(num_tracks)
                .collect(),
        }
    }
}

/// Messages sent from the server to a client.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMessage {
    /// Sent immediately on connect: the client's assigned ID and the full state snapshot.
    Init {
        client_id: ClientId,
        state: SequencerState,
    },
    /// A single track's state has changed (steps toggled, lock acquired, or lock released).
    /// Always carries the full new [`TrackState`] so clients stay consistent.
    TrackUpdate { track: u32, state: TrackState },
    /// Sent only to the requesting client when the track is already locked by someone else.
    LockDenied { track: u32 },
}

/// Commands sent from a client to the server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientCommand {
    /// Request exclusive edit access to a track.
    RequestLock { track: u32 },
    /// Relinquish edit access to a track.
    ReleaseLock { track: u32 },
    /// Toggle a step. Silently ignored by the server if the caller does not hold the lock.
    ToggleStep { track: u32, step: u32 },
}
