use crate::frontend::state::State;

/// Renders the top bar. Returns `true` if the Back button was clicked.
pub fn show_top_panel(state: &mut State, ui: &mut egui::Ui) -> bool {
    let mut back_clicked = false;
    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
        match state {
            State::Connected(_, ui_state) => {
                if ui.add(egui::Button::new("Back")).clicked() {
                    ui_state.back();
                    back_clicked = true;
                }
            }
            State::Disconnected(_) => {}
        }
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.label(state.summary_string());
            },
        );
    });
    back_clicked
}
