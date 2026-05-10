use crate::frontend::state::{State, UiState};
use crate::frontend::view_ctx::ViewCtx;

use super::player_selection::show_player_selection;
use super::track_view::show_track_view;

/// Routes to the correct view based on navigation state.
///
/// Sets `lock_request` when the user has just navigated to a new track,
/// so the caller can emit `RequestLock` after the borrow of `state` ends.
pub fn show_central_panel(
    vctx: &mut ViewCtx,
    state: &mut State,
    ui: &mut egui::Ui,
    lock_request: &mut Option<u32>,
) {
    let selected_track: Option<usize> = match state {
        State::Disconnected(msg) => {
            ui.label(msg.clone());
            None
        }
        State::Connected(_, ui_state) => match ui_state {
            UiState::PlayerSelection => {
                show_player_selection(ui_state, ui);
                // If the player selection navigated to a track this frame,
                // signal that a lock should be requested.
                if let Some(idx) = ui_state.track_index() {
                    *lock_request = Some(idx as u32);
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
        show_track_view(vctx, ui, track_idx);
    }
}
