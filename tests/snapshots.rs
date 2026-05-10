use libretakt::frontend::app_state::WsToUiMsg;
use libretakt::frontend::{app_state::create_channels, LibretaktUI};
use libretakt::shared::{ClientId, SequencerState, ServerMessage};

/// Build a phone-sized harness with the given app, run one frame so that all
/// queued `WsToUiMsg` messages are processed, then return the harness ready
/// for snapshotting.
fn phone_harness(app: LibretaktUI) -> egui_kittest::Harness<'static, LibretaktUI> {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(390.0, 844.0))
        .build_ui_state(
            |ui: &mut egui::Ui, libretakt_ui: &mut LibretaktUI| {
                libretakt_ui.render(ui.ctx());
            },
            app,
        );
    harness.run();
    harness
}

#[test]
fn hello_world_snapshot() {
    let mut harness = egui_kittest::Harness::new_ui(|ui| {
        ui.label("Hello, world!");
        let _ = ui.button("Click me");
    });

    harness.snapshot("hello_world");
}

#[test]
fn app_default_view_snapshot() {
    let (app_state, _ws_channels) = create_channels();
    let app = LibretaktUI::new_for_test(app_state);

    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(390.0, 844.0)) // portrait phone viewport
        .build_ui_state(
            |ui: &mut egui::Ui, libretakt_ui: &mut LibretaktUI| {
                libretakt_ui.render(ui.ctx());
            },
            app,
        );

    harness.snapshot("app_default_view");
}

/// After a `ServerMessage::Init` the UI should transition out of the
/// "Connecting..." state and show the player-selection screen.
#[test]
fn app_connected_player_selection_snapshot() {
    let (app_state, ws_channels) = create_channels();

    // Simulate the server sending an Init message to the client.
    ws_channels
        .to_ui
        .unbounded_send(WsToUiMsg::Server(ServerMessage::Init {
            client_id: 1,
            state: SequencerState::new(8, 16),
        }))
        .unwrap();

    let app = LibretaktUI::new_for_test(app_state);
    let mut harness = phone_harness(app);
    harness.snapshot("app_connected_player_selection");
}

/// One track locked by this client, one locked by a peer, the rest free.
/// Useful for checking that lock indicators render correctly.
#[test]
fn app_track_lock_states_snapshot() {
    const MY_ID: ClientId = 42;
    const PEER_ID: ClientId = 7;

    let (mut app_state, ws_channels) = create_channels();

    // Pre-populate sequencer with 4 tracks so we can control lock state.
    let mut sequencer = SequencerState::new(4, 16);
    sequencer.tracks[0].locked_by = Some(MY_ID); // locked by me
    sequencer.tracks[1].locked_by = Some(PEER_ID); // locked by someone else
                                                   // tracks[2] and [3] remain free
    app_state.sequencer = sequencer;

    // Send Init so the UI moves to the connected/player-selection view
    // and picks up our client ID.
    ws_channels
        .to_ui
        .unbounded_send(WsToUiMsg::Server(ServerMessage::Init {
            client_id: MY_ID,
            state: app_state.sequencer.clone(),
        }))
        .unwrap();

    let app = LibretaktUI::new_for_test(app_state);
    let mut harness = phone_harness(app);
    harness.snapshot("app_track_lock_states");
}

/// After a disconnection event the UI should display a reconnecting message.
#[test]
fn app_disconnected_snapshot() {
    let (app_state, ws_channels) = create_channels();

    // First connect, then immediately disconnect, so we exercise the
    // transition rather than just the initial "Connecting..." splash.
    ws_channels
        .to_ui
        .unbounded_send(WsToUiMsg::Server(ServerMessage::Init {
            client_id: 1,
            state: SequencerState::new(8, 16),
        }))
        .unwrap();
    ws_channels
        .to_ui
        .unbounded_send(WsToUiMsg::Disconnected)
        .unwrap();

    let app = LibretaktUI::new_for_test(app_state);
    let mut harness = phone_harness(app);
    harness.snapshot("app_disconnected");
}
