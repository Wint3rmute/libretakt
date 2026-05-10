use crate::frontend::state::UiState;

pub fn show_player_selection(ui_state: &mut UiState, ui: &mut egui::Ui) {
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
