mod main;
pub use main::main;

pub mod app_state;
mod notifications;
mod sequencer;
mod state;
mod view_ctx;
mod views;

use app_state::{ApplicationState, WsToUiMsg};
use notifications::NotificationQueue;
use sequencer::LocalSequencerState;
use state::{ProjectData, State, UiState};
use view_ctx::ViewCtx;
use views::{show_bottom_panel, show_central_panel, show_top_panel};

use crate::shared::{ClientCommand, ServerMessage};

pub struct LibretaktUI {
    state: State,
    local_seq: LocalSequencerState,
    app_state: ApplicationState,
    notifications: NotificationQueue,
    /// Local-only parameter values for each track: [filter, resonance, volume, pan].
    track_params: Vec<[f32; 4]>,
    /// Commands queued during rendering, flushed to the WebSocket at the end of each frame.
    outbox: Vec<ClientCommand>,
}

impl LibretaktUI {
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

    /// Drain incoming WebSocket messages and update application state.
    fn process_inbound(&mut self) {
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
    }

    /// Render invisible side panels that center the content in a 9:16 portrait column.
    fn render_margins(&self, ctx: &egui::Context) {
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
    }

    /// Release every track lock held by this client.
    fn release_all_locks(&mut self) {
        let client_id = self.app_state.client_id;
        for (idx, track) in self.app_state.sequencer.tracks.iter().enumerate() {
            if track.locked_by == Some(client_id) {
                self.outbox
                    .push(ClientCommand::ReleaseLock { track: idx as u32 });
            }
        }
    }

    /// Send all queued commands over the WebSocket.
    fn flush_outbox(&mut self) {
        for cmd in self.outbox.drain(..) {
            self.app_state.to_ws.unbounded_send(cmd).ok();
        }
    }
}

impl eframe::App for LibretaktUI {
    fn ui(&mut self, _ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // All rendering is handled in `update` via explicit panel layout.
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Phase 1 — Inbound ──────────────────────────────────────────────
        self.process_inbound();

        // ── Phase 2 — Render ───────────────────────────────────────────────
        self.render_margins(ctx);

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            show_bottom_panel(&mut self.notifications, ctx, ui);
        });

        let back_clicked = egui::TopBottomPanel::top("top_panel")
            .show(ctx, |ui| show_top_panel(&mut self.state, ui))
            .inner;

        let mut lock_request: Option<u32> = None;
        {
            // Construct ViewCtx with explicit field borrows so that self.state
            // remains unborrowed and can be passed to show_central_panel.
            let mut vctx = ViewCtx {
                app_state: &self.app_state,
                outbox: &mut self.outbox,
                notifications: &mut self.notifications,
                track_params: &mut self.track_params,
                local_seq: &self.local_seq,
            };
            egui::CentralPanel::default().show(ctx, |ui| {
                show_central_panel(&mut vctx, &mut self.state, ui, &mut lock_request);
            });
        }

        if back_clicked {
            self.release_all_locks();
        }
        if let Some(track) = lock_request {
            self.outbox.push(ClientCommand::RequestLock { track });
        }

        // ── Phase 3 — Outbound ─────────────────────────────────────────────
        self.flush_outbox();
    }
}
