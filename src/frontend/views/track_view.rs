use crate::frontend::view_ctx::ViewCtx;
use crate::shared::{ClientCommand, TrackState};

/// Top-level entry point for the single-track sequencer view.
pub fn show_track_view(vctx: &mut ViewCtx, ui: &mut egui::Ui, track_idx: usize) {
    let Some(track_state) = vctx.app_state.sequencer.tracks.get(track_idx).cloned() else {
        ui.centered_and_justified(|ui| {
            ui.label(format!("Track {} not found", track_idx + 1));
        });
        return;
    };

    show_params(vctx, ui, track_idx);
    ui.separator();
    show_step_grid(vctx, ui, track_idx, &track_state);
}

/// Parameter sliders for the selected track (local-only prototype).
fn show_params(vctx: &mut ViewCtx, ui: &mut egui::Ui, track_idx: usize) {
    let n = 4.0_f32;
    let item_spacing = ui.spacing().item_spacing.y;
    let label_height = ui.text_style_height(&egui::TextStyle::Small);
    let slider_height =
        ((ui.available_height() / 3.0 - item_spacing * (n - 1.0)) / n).clamp(32.0, 64.0);
    let params_height =
        n * (label_height + item_spacing + slider_height) + (n - 1.0) * item_spacing;

    ui.allocate_ui(egui::Vec2::new(ui.available_width(), params_height), |ui| {
        let params = &mut vctx.track_params[track_idx];
        let width = ui.available_width();
        ui.spacing_mut().slider_width = (width - 60.0).max(60.0);
        ui.spacing_mut().interact_size.y = slider_height / 2.0;
        ui.vertical(|ui| {
            for (value, label) in params
                .iter_mut()
                .zip(["Filter", "Resonance", "Volume", "Pan"])
            {
                ui.label(label);
                ui.add(egui::Slider::new(value, 0.0..=1.0).text(""));
            }
        });
    });
}

/// 4×4 step grid for the selected track.
fn show_step_grid(
    vctx: &mut ViewCtx,
    ui: &mut egui::Ui,
    track_idx: usize,
    track_state: &TrackState,
) {
    let my_id = vctx.app_state.client_id;
    let i_own_lock = track_state.locked_by == Some(my_id);
    let current_step = vctx.local_seq.current_step;

    let spacing = 4.0;
    let step_size = ((ui.available_width() - 3.0 * spacing) / 4.0).min(120.0);
    let step_size = egui::Vec2::splat(step_size);

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

                    let label = egui::RichText::new(format!("{}", step_idx + 1)).color(text_color);
                    let resp = ui.add_sized(step_size, egui::Button::new(label).fill(fill));

                    if resp.clicked() && i_own_lock {
                        vctx.outbox.push(ClientCommand::ToggleStep {
                            track: track_idx as u32,
                            step: step_idx as u32,
                        });
                    }
                }
                ui.end_row();
            }
        });
}
