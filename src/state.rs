pub struct ProjectData;

pub enum UiState {
    PlayerSelection,
    MixingConsole_T0,
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
    pub fn summary(&self) -> String {
        match &self {
            UiState::PlayerSelection => "Player Selection",
            UiState::MixingConsole_T0 => "Mixing Console",
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
