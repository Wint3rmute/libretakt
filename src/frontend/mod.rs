mod main;
pub use main::main;

pub mod app_state;
mod sequencer;
mod state;

use app_state::ApplicationState;
use egui::{Context, Ui};
use sequencer::LocalSequencerState;
use state::{ProjectData, State, UiState};

use crate::shared::{ClientCommand, ServerMessage};

pub struct LibretaktUI {
    state: State,
    local_seq: LocalSequencerState,
    app_state: ApplicationState,
    /// Commands queued during rendering, flushed to the WebSocket at the end of each frame.
    outbox: Vec<ClientCommand>,
}

impl LibretaktUI {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>, app_state: ApplicationState) -> Self {
        tracing::info!("Creating UI...");
        Self {
            state: State::Disconnected("Connecting...".to_string()),
            local_seq: LocalSequencerState::default(),
            app_state,
            outbox: Vec::new(),
        }
    }

    /// Render the sequencer grid for all tracks.
    ///
    /// Clones the track list up front so that `self.outbox` can be borrowed
    /// mutably within the loop without conflicting with `self.app_state`.
    fn show_sequencer(&mut self, _ctx: &Context, ui: &mut Ui) {
        let my_id = self.app_state.client_id;
        // Clone tracks to avoid a simultaneous immutable borrow of app_state
        // and a mutable borrow of outbox inside the loop.
        let tracks: Vec<_> = self
            .app_state
            .sequencer
            .tracks
            .iter()
            .cloned()
            .enumerate()
            .collect();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (track_idx, track_state) in tracks {
                ui.horizontal(|ui| {
                    let i_own_lock = track_state.locked_by == Some(my_id);
                    let is_locked_by_other = track_state.locked_by.is_some() && !i_own_lock;

                    // --- Lock button (60×36 px) ---
                    let (lock_label, lock_fill) = if i_own_lock {
                        ("🔒 Mine", egui::Color32::DARK_GREEN)
                    } else if is_locked_by_other {
                        ("🔒 Busy", egui::Color32::DARK_RED)
                    } else {
                        ("🔓 Free", egui::Color32::TRANSPARENT)
                    };

                    let lock_resp =
                        ui.add_sized([60.0, 36.0], egui::Button::new(lock_label).fill(lock_fill));

                    if lock_resp.clicked() {
                        if i_own_lock {
                            self.outbox.push(ClientCommand::ReleaseLock {
                                track: track_idx as u32,
                            });
                        } else if !is_locked_by_other {
                            self.outbox.push(ClientCommand::RequestLock {
                                track: track_idx as u32,
                            });
                        }
                        // locked_by == Some(other_id): button visible but does nothing
                    }

                    // --- Step buttons (36×36 px each) ---
                    let current_step = self.local_seq.current_step;
                    let text_color = if i_own_lock {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::GRAY
                    };

                    for (step_idx, &active) in track_state.steps.iter().enumerate() {
                        let fill = if active && step_idx == current_step {
                            egui::Color32::LIGHT_GREEN
                        } else if active {
                            egui::Color32::DARK_GREEN
                        } else if step_idx == current_step {
                            egui::Color32::DARK_GRAY
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let step_text =
                            egui::RichText::new(format!("{}", step_idx + 1)).color(text_color);

                        let step_resp =
                            ui.add_sized([36.0, 36.0], egui::Button::new(step_text).fill(fill));

                        if step_resp.clicked() && i_own_lock {
                            self.outbox.push(ClientCommand::ToggleStep {
                                track: track_idx as u32,
                                step: step_idx as u32,
                            });
                        }
                    }
                });
            }
        });
    }
}

impl eframe::App for LibretaktUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Phase 1 — Inbound ──────────────────────────────────────────────
        // Drain all available WebSocket messages and mutate app_state.
        while let Ok(Some(msg)) = self.app_state.from_ws.try_next() {
            match msg {
                ServerMessage::Init { client_id, state } => {
                    tracing::info!("Received Init as client {}", client_id);
                    self.app_state.client_id = client_id;
                    self.app_state.sequencer = state;
                    self.state = State::Connected(ProjectData, UiState::PlayerSelection);
                }
                ServerMessage::TrackUpdate { track, state } => {
                    if let Some(t) = self.app_state.sequencer.tracks.get_mut(track as usize) {
                        *t = state;
                    }
                }
                ServerMessage::LockDenied { track } => {
                    tracing::warn!("Lock denied for track {}", track);
                    self.app_state.lock_denied_track = Some(track as usize);
                }
            }
        }

        // ── Phase 2 — Render ───────────────────────────────────────────────

        // Bottom status bar: connection state + transient lock-denied notice.
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            let conn_str = match &self.state {
                State::Connected(_, _) => "Connected".to_string(),
                State::Disconnected(msg) => format!("Disconnected: {msg}"),
            };
            // .take() clears the flag after one frame so the notice is a brief flash.
            let status = if let Some(track) = self.app_state.lock_denied_track.take() {
                format!("{conn_str} | ⚠ Lock denied for track {}", track + 1)
            } else {
                conn_str
            };
            ui.label(status);
        });

        // Top panel: "Back" button + state summary label.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                // Mutable borrow of self.state released at the end of this match.
                match &mut self.state {
                    State::Connected(_, ui_state) => {
                        if ui.add(egui::Button::new("Back")).clicked() {
                            ui_state.back();
                        }
                    }
                    State::Disconnected(_) => {}
                }

                // Immutable borrow of self.state; no overlap with the match above.
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.label(self.state.summary_string());
                    },
                );
            });
        });

        // Central panel: player selection, sequencer, or disconnected notice.
        egui::CentralPanel::default().show(ctx, |ui| {
            // Determine what to render. The mutable borrow of self.state ends
            // when the match expression is done, before show_sequencer is called.
            let show_seq = match &mut self.state {
                State::Disconnected(msg) => {
                    ui.label(msg.clone());
                    false
                }
                State::Connected(_, ui_state) => match ui_state {
                    UiState::PlayerSelection => {
                        show_player_selection(ui_state, ctx, ui);
                        false
                    }
                    // All non-selection views currently show the collaborative sequencer.
                    UiState::AudioTrackT1
                    | UiState::AudioTrackT2
                    | UiState::AudioTrackT3
                    | UiState::AudioTrackT4
                    | UiState::AudioTrackT5
                    | UiState::AudioTrackT6
                    | UiState::AudioTrackT7
                    | UiState::AudioTrackT8
                    | UiState::MixingConsoleT0 => true,
                },
            };

            if show_seq {
                self.show_sequencer(ctx, ui);
            }
        });

        // ── Phase 3 — Outbound ─────────────────────────────────────────────
        // Serialise and send every command that was queued during rendering.
        for cmd in self.outbox.drain(..) {
            self.app_state.to_ws.unbounded_send(cmd).ok();
        }
    }
}

/// Render the player / view selection screen.
///
/// Every `UiState` variant that can be navigated to must be constructed here
/// so the compiler considers it "used" (required for `-D dead-code`).
fn show_player_selection(ui_state: &mut UiState, _ctx: &Context, ui: &mut Ui) {
    let w = ui.min_size().x / 2.0;
    let h = 60.0;
    let c = egui::Color32::TRANSPARENT;

    egui::Grid::new("player_selection_id")
        .spacing(egui::Vec2::ZERO)
        .show(ui, |ui| {
            if ui
                .add_sized([w, h], egui::Button::new("Sequencer").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT1;
            }
            if ui
                .add_sized([w, h], egui::Button::new("Track 2").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT2;
            }
            ui.end_row();
            if ui
                .add_sized([w, h], egui::Button::new("Track 3").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT3;
            }
            if ui
                .add_sized([w, h], egui::Button::new("Track 4").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT4;
            }
            ui.end_row();
            if ui
                .add_sized([w, h], egui::Button::new("Track 5").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT5;
            }
            if ui
                .add_sized([w, h], egui::Button::new("Track 6").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT6;
            }
            ui.end_row();
            if ui
                .add_sized([w, h], egui::Button::new("Track 7").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT7;
            }
            if ui
                .add_sized([w, h], egui::Button::new("Track 8").fill(c))
                .clicked()
            {
                *ui_state = UiState::AudioTrackT8;
            }
            ui.end_row();
            if ui
                .add_sized([w, h], egui::Button::new("Mixing Console").fill(c))
                .clicked()
            {
                *ui_state = UiState::MixingConsoleT0;
            }
        });
}
