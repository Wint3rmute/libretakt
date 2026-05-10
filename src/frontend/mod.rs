mod main;
pub use main::main;

pub mod app_state;
mod notifications;
mod sequencer;
mod state;

use app_state::{ApplicationState, WsToUiMsg};
use egui::{Context, Ui};
use notifications::NotificationQueue;
use sequencer::LocalSequencerState;
use state::{ProjectData, State, UiState};

use crate::shared::{ClientCommand, ServerMessage};

pub struct LibretaktUI {
    state: State,
    local_seq: LocalSequencerState,
    app_state: ApplicationState,
    notifications: NotificationQueue,
    /// Local-only parameter values for each track: [filter, resonance, volume, pan].
    /// Indexed by track index; not synced to the server.
    track_params: Vec<[f32; 4]>,
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
            notifications: NotificationQueue::default(),
            track_params: vec![[0.5; 4]; 8],
            outbox: Vec::new(),
        }
    }

    /// Render the single-track sequencer view for `track_idx`.
    ///
    /// Layout:
    ///   * Top half  - lock button + reserved space for track parameters.
    ///   * Bottom half - 4x4 step grid, sized to fill the available width.
    fn show_sequencer(&mut self, _ctx: &Context, ui: &mut Ui, track_idx: usize) {
        let Some(track_state) = self.app_state.sequencer.tracks.get(track_idx).cloned() else {
            ui.centered_and_justified(|ui| {
                ui.label(format!("Track {} not found", track_idx + 1));
            });
            return;
        };

        let my_id = self.app_state.client_id;
        let i_own_lock = track_state.locked_by == Some(my_id);
        let _is_locked_by_other = track_state.locked_by.is_some() && !i_own_lock;

        // -- Top half: parameter sliders (local-only prototype) ---------------
        let params_height = ui.available_height() * 0.5;
        ui.allocate_ui(egui::Vec2::new(ui.available_width(), params_height), |ui| {
            let params = &mut self.track_params[track_idx];
            let width = ui.available_width();
            let n = 4.0_f32;
            let slider_height =
                (ui.available_height() - ui.spacing().item_spacing.y * (n - 1.0)) / n;
            ui.vertical(|ui| {
                for (value, label) in
                    params
                        .iter_mut()
                        .zip(["Filter", "Resonance", "Volume", "Pan"])
                {
                    ui.add_sized(
                        [width, slider_height],
                        egui::Slider::new(value, 0.0..=1.0).text(label),
                    );
                }
            });
        });

        ui.separator();

        // -- Bottom half: 4x4 step grid -------------------------------------
        let spacing = 4.0;
        let step_size = ((ui.available_width() - 3.0 * spacing) / 4.0).min(120.0);
        let step_size = egui::Vec2::splat(step_size);
        let current_step = self.local_seq.current_step;

        egui::Grid::new(format!("steps_{track_idx}"))
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for row in 0..4_usize {
                    for col in 0..4_usize {
                        let step_idx = row * 4 + col;
                        let active = track_state.steps.get(step_idx).copied().unwrap_or(false);

                        let fill = if active && step_idx == current_step {
                            egui::Color32::LIGHT_GREEN
                        } else if active {
                            egui::Color32::DARK_GREEN
                        } else if step_idx == current_step {
                            egui::Color32::DARK_GRAY
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let text_color = if i_own_lock {
                            ui.visuals().strong_text_color()
                        } else {
                            ui.visuals().weak_text_color()
                        };

                        let label =
                            egui::RichText::new(format!("{}", step_idx + 1)).color(text_color);
                        let resp = ui.add_sized(step_size, egui::Button::new(label).fill(fill));

                        if resp.clicked() && i_own_lock {
                            self.outbox.push(ClientCommand::ToggleStep {
                                track: track_idx as u32,
                                step: step_idx as u32,
                            });
                        }
                    }
                    ui.end_row();
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
                WsToUiMsg::Disconnected => {
                    tracing::warn!("Disconnected from server");
                    self.state = State::Disconnected("Reconnecting...".to_string());
                    self.notifications.push("Disconnected, reconnecting...");
                }
                WsToUiMsg::Server(ServerMessage::Init { client_id, state }) => {
                    tracing::info!("Received Init as client {}", client_id);
                    self.app_state.client_id = client_id;
                    self.app_state.sequencer = state;
                    self.state = State::Connected(ProjectData, UiState::PlayerSelection);
                    self.notifications.push("Connected");
                }
                WsToUiMsg::Server(ServerMessage::TrackUpdate { track, state }) => {
                    if let Some(t) = self.app_state.sequencer.tracks.get_mut(track as usize) {
                        *t = state;
                    }
                }
                WsToUiMsg::Server(ServerMessage::LockDenied { track }) => {
                    tracing::warn!("Lock denied for track {}", track);
                    self.notifications
                        .push(format!("Lock denied for track {}", track + 1));
                }
            }
        }

        // ── Phase 2 — Render ───────────────────────────────────────────────

        // Center the UI in a 9:16 portrait column so it looks natural on both
        // mobile (margin == 0, panels fill the screen) and desktop (equal
        // margins push all panels into the center).
        let viewport = ctx.screen_rect();
        let content_width = (viewport.height() * (9.0 / 16.0)).min(viewport.width());
        let h_margin = ((viewport.width() - content_width) / 2.0).max(0.0);
        if h_margin > 0.0 {
            let margin_frame = egui::Frame::none().fill(ctx.style().visuals.panel_fill);
            egui::SidePanel::left("left_margin")
                .exact_width(h_margin)
                .resizable(false)
                .frame(margin_frame)
                .show(ctx, |_ui| {});
            egui::SidePanel::right("right_margin")
                .exact_width(h_margin)
                .resizable(false)
                .frame(margin_frame)
                .show(ctx, |_ui| {});
        }

        // Bottom status bar: notifications only.
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            let now = ctx.input(|i| i.time);
            if let Some((msg, alpha)) = self.notifications.current(now) {
                let base = ui.visuals().text_color();
                let color = egui::Color32::from_rgba_unmultiplied(
                    base.r(),
                    base.g(),
                    base.b(),
                    (alpha * 255.0) as u8,
                );
                ui.colored_label(color, msg);
                ctx.request_repaint(); // keep animating while a notification is visible
            } else {
                ui.label(""); // maintain consistent panel height
            }
        });

        // Top panel: "Back" button + state summary label.
        let mut back_clicked = false;
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                // Mutable borrow of self.state released at the end of this match.
                match &mut self.state {
                    State::Connected(_, ui_state) => {
                        if ui.add(egui::Button::new("Back")).clicked() {
                            ui_state.back();
                            back_clicked = true;
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
        let mut lock_request: Option<u32> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            // Resolve which track to display before calling show_sequencer, so
            // that the borrow of self.state ends before the mutable borrow in
            // show_sequencer begins.
            let selected_track: Option<usize> = match &mut self.state {
                State::Disconnected(msg) => {
                    ui.label(msg.clone());
                    None
                }
                State::Connected(_, ui_state) => match ui_state {
                    UiState::PlayerSelection => {
                        show_player_selection(ui_state, ctx, ui);
                        // If show_player_selection navigated to a track, request its lock.
                        if let Some(idx) = ui_state.track_index() {
                            lock_request = Some(idx as u32);
                        }
                        ui_state.track_index()
                    }
                    UiState::MixingConsoleT0 => {
                        ui.centered_and_justified(|ui| {
                            ui.label("Mixing Console - coming soon");
                        });
                        None
                    }
                    other => other.track_index(),
                },
            };

            if let Some(track_idx) = selected_track {
                self.show_sequencer(ctx, ui, track_idx);
            }
        });

        // Release all owned locks when Back is pressed.
        if back_clicked {
            let client_id = self.app_state.client_id;
            for (idx, track) in self.app_state.sequencer.tracks.iter().enumerate() {
                if track.locked_by == Some(client_id) {
                    self.outbox
                        .push(ClientCommand::ReleaseLock { track: idx as u32 });
                }
            }
        }

        // Request lock for a newly selected track.
        if let Some(track) = lock_request {
            self.outbox.push(ClientCommand::RequestLock { track });
        }

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
