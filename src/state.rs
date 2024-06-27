pub struct ProjectData;

pub enum UiState {
    PlayerSelection,
    MixingConsole_T0,
    AudioTrack_T1,
    AudioTrack_T2,
    AudioTrack_T3,
    AudioTrack_T4,
    AudioTrack_T5,
    AudioTrack_T6,
    AudioTrack_T7,
    AudioTrack_T8,
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
            UiState::AudioTrack_T1 => "Track 1",
            UiState::AudioTrack_T2 => "Track 2",
            UiState::AudioTrack_T3 => "Track 3",
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
