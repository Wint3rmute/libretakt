use futures_channel::mpsc;

use crate::shared::{ClientCommand, ClientId, SequencerState, ServerMessage};

/// Messages sent from the WebSocket background task to the UI.
pub type WsToUiMsg = ServerMessage;

/// Messages sent from the UI to the WebSocket background task.
pub type UiToWsMsg = ClientCommand;

/// Shared application state passed into the UI at startup.
pub struct ApplicationState {
    /// Receive server messages arriving from the WebSocket background task.
    pub from_ws: mpsc::UnboundedReceiver<WsToUiMsg>,

    /// Send commands out to the WebSocket background task.
    pub to_ws: mpsc::UnboundedSender<UiToWsMsg>,

    /// This client's ID, assigned by the server in the `Init` message.
    pub client_id: ClientId,

    /// The authoritative sequencer state, kept in sync with the server via
    /// `ServerMessage::Init` and `ServerMessage::TrackUpdate` messages.
    pub sequencer: SequencerState,
}

/// Companion channels that belong to the WebSocket background task.
pub struct WsChannels {
    /// Forward incoming WebSocket messages to the UI.
    pub to_ui: mpsc::UnboundedSender<WsToUiMsg>,

    /// Receive commands from the UI to send over the WebSocket.
    pub from_ui: mpsc::UnboundedReceiver<UiToWsMsg>,
}

/// Create a linked `(ApplicationState, WsChannels)` pair.
///
/// `ApplicationState` is owned by the UI task; `WsChannels` is owned by the
/// WebSocket background task. Together they form two `mpsc` channels that
/// bridge the two tasks.
pub fn create_channels() -> (ApplicationState, WsChannels) {
    let (ws_to_ui_tx, ws_to_ui_rx) = mpsc::unbounded();
    let (ui_to_ws_tx, ui_to_ws_rx) = mpsc::unbounded();

    let app_state = ApplicationState {
        from_ws: ws_to_ui_rx,
        to_ws: ui_to_ws_tx,
        client_id: 0,
        sequencer: SequencerState::default(),
    };

    let ws_channels = WsChannels {
        to_ui: ws_to_ui_tx,
        from_ui: ui_to_ws_rx,
    };

    (app_state, ws_channels)
}
