pub struct ProjectData;

pub enum UiState {
    PlayerSelection,
    MixingConsoleT0,
    AudioTrackT1,
    AudioTrackT2,
    AudioTrackT3,
    AudioTrackT4,
    AudioTrackT5,
    AudioTrackT6,
    AudioTrackT7,
    AudioTrackT8,
}

pub enum State {
    Connected(ProjectData, UiState),
    Disconnected(String),
}

impl State {
    pub fn summary_string(&self) -> String {
        match &self {
            State::Disconnected(error_message) => format!("Disconnected: {error_message}"),
            State::Connected(_project_data, ui_state) => ui_state.summary(),
        }
    }
}

impl UiState {
    /// Returns the sequencer track index this view corresponds to, or `None`
    /// for views that do not display a single track (e.g. PlayerSelection,
    /// MixingConsole).
    pub fn track_index(&self) -> Option<usize> {
        match self {
            UiState::PlayerSelection | UiState::MixingConsoleT0 => None,
            UiState::AudioTrackT1 => Some(0),
            UiState::AudioTrackT2 => Some(1),
            UiState::AudioTrackT3 => Some(2),
            UiState::AudioTrackT4 => Some(3),
            UiState::AudioTrackT5 => Some(4),
            UiState::AudioTrackT6 => Some(5),
            UiState::AudioTrackT7 => Some(6),
            UiState::AudioTrackT8 => Some(7),
        }
    }

    pub fn summary(&self) -> String {
        match &self {
            UiState::PlayerSelection => "Player Selection",
            UiState::MixingConsoleT0 => "Mixing Console",
            UiState::AudioTrackT1 => "Track 1",
            UiState::AudioTrackT2 => "Track 2",
            UiState::AudioTrackT3 => "Track 3",
            _ => "TODO: undefined status summary",
        }
        .to_string()
    }
    pub fn back(&mut self) {
        match &self {
            UiState::PlayerSelection => {}
            _ => {
                *self = UiState::PlayerSelection;
            }
        }
    }
}
