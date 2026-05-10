use egui_kittest::kittest::Queryable as _;
use libretakt::frontend::app_state::WsToUiMsg;
use libretakt::frontend::{app_state::create_channels, LibretaktUI};
use libretakt::shared::{ClientId, SequencerState, ServerMessage};

/// Render `make_app` in both dark and light themes and save a snapshot for each.
///
/// `make_app` is called twice — once per theme — so it must produce a fresh
/// `LibretaktUI` each time (required because `ApplicationState` owns the
/// channel receiver and cannot be cloned or reused).
///
/// Snapshot files are written as `{name}_dark.png` and `{name}_light.png`.
fn snapshot_themes(name: &str, make_app: impl FnMut() -> LibretaktUI) {
    snapshot_themes_interact(name, make_app, |_| {});
}

/// Like [`snapshot_themes`], but runs `interact` on the harness between
/// construction and snapshotting. Use this to simulate user navigation
/// (e.g. clicking a button to open a view) before taking the screenshot.
fn snapshot_themes_interact(
    name: &str,
    mut make_app: impl FnMut() -> LibretaktUI,
    mut interact: impl FnMut(&mut egui_kittest::Harness<LibretaktUI>),
) {
    // Collect results from both harnesses into one before asserting, as
    // required by egui_kittest when multiple harnesses exist in a single test.
    let mut all_results = egui_kittest::SnapshotResults::new();

    for (theme, suffix) in [(egui::Theme::Dark, "dark"), (egui::Theme::Light, "light")] {
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::Vec2::new(390.0, 844.0))
            .with_theme(theme)
            .build_ui_state(
                |ui: &mut egui::Ui, app: &mut LibretaktUI| {
                    app.tick(ui.ctx());
                },
                make_app(),
            );
        interact(&mut harness);
        harness.step(); // apply any queued interactions
        harness.snapshot(&format!("{name}_{suffix}"));
        all_results.extend(harness.take_snapshot_results());
    }

    // Panics here (on drop) if any snapshot mismatched or was missing.
    drop(all_results);
}

#[test]
fn hello_world_snapshot() {
    let mut harness = egui_kittest::Harness::new_ui(|ui| {
        ui.label("Hello, world!");
        let _ = ui.button("Click me");
    });

    harness.snapshot("hello_world");
}

/// Initial state before the server has replied: shows the "Connecting…" splash.
#[test]
fn app_default_view_snapshot() {
    snapshot_themes("app_default_view", || {
        let (app_state, _ws_channels) = create_channels();
        LibretaktUI::new_for_test(app_state)
    });
}

/// After a `ServerMessage::Init` the UI should transition out of the
/// "Connecting..." state and show the player-selection screen.
#[test]
fn app_connected_player_selection_snapshot() {
    snapshot_themes("app_connected_player_selection", || {
        let (app_state, ws_channels) = create_channels();
        ws_channels
            .to_ui
            .unbounded_send(WsToUiMsg::Server(ServerMessage::Init {
                client_id: 1,
                state: SequencerState::new(8, 16),
            }))
            .unwrap();
        LibretaktUI::new_for_test(app_state)
    });
}

/// One track locked by this client, one locked by a peer, the rest free.
/// Useful for checking that lock indicators render correctly.
#[test]
fn app_track_lock_states_snapshot() {
    const MY_ID: ClientId = 42;
    const PEER_ID: ClientId = 7;

    snapshot_themes("app_track_lock_states", || {
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

        LibretaktUI::new_for_test(app_state)
    });
}

/// After a disconnection event the UI should display a reconnecting message.
#[test]
fn app_disconnected_snapshot() {
    snapshot_themes("app_disconnected", || {
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

        LibretaktUI::new_for_test(app_state)
    });
}

/// The track-1 sequencer view, with a few active steps and the lock held by
/// this client. Exercises the step grid, parameter sliders, and lock styling.
#[test]
fn app_sequencer_view_snapshot() {
    const MY_ID: ClientId = 1;

    snapshot_themes_interact(
        "app_sequencer_view",
        || {
            let (app_state, ws_channels) = create_channels();

            let mut sequencer = SequencerState::new(8, 16);
            // Activate a simple four-on-the-floor pattern on track 1.
            for step in [0, 4, 8, 12] {
                sequencer.tracks[0].steps[step] = true;
            }
            // This client holds the lock on track 1.
            sequencer.tracks[0].locked_by = Some(MY_ID);

            ws_channels
                .to_ui
                .unbounded_send(WsToUiMsg::Server(ServerMessage::Init {
                    client_id: MY_ID,
                    state: sequencer,
                }))
                .unwrap();

            LibretaktUI::new_for_test(app_state)
        },
        |harness| {
            // Navigate from PlayerSelection into the track-1 sequencer view.
            harness.get_by_label("Sequencer").click();
        },
    );
}
