use futures_channel::mpsc;

/// Messages sent from the WebSocket background task to the UI.
pub type WsToUiMsg = String;

/// Messages sent from the UI to the WebSocket background task.
pub type UiToWsMsg = String;

/// Shared application state passed into the UI at startup.
pub struct ApplicationState {
    /// Receive updates arriving from the WebSocket background task.
    pub from_ws: mpsc::UnboundedReceiver<WsToUiMsg>,

    /// Send commands out to the WebSocket background task.
    pub to_ws: mpsc::UnboundedSender<UiToWsMsg>,

    /// Example state variable: last status message from the server.
    pub server_status: String,
}

/// Companion channels that belong to the WebSocket background task.
pub struct WsChannels {
    /// Forward incoming WebSocket messages to the UI.
    pub to_ui: mpsc::UnboundedSender<WsToUiMsg>,

    /// Receive commands from the UI to send over the WebSocket.
    pub from_ui: mpsc::UnboundedReceiver<UiToWsMsg>,
}

/// Create a linked `(ApplicationState, WsChannels)` pair.
pub fn create_channels() -> (ApplicationState, WsChannels) {
    let (ws_to_ui_tx, ws_to_ui_rx) = mpsc::unbounded();
    let (ui_to_ws_tx, ui_to_ws_rx) = mpsc::unbounded();

    let app_state = ApplicationState {
        from_ws: ws_to_ui_rx,
        to_ws: ui_to_ws_tx,
        server_status: "Connecting...".to_string(),
    };

    let ws_channels = WsChannels {
        to_ui: ws_to_ui_tx,
        from_ui: ui_to_ws_rx,
    };

    (app_state, ws_channels)
}
