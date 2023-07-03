use egui::Align;
use log::info;

pub struct LibretaktUI {}

impl Default for LibretaktUI {
    fn default() -> Self {
        Self {}
    }
}

impl LibretaktUI {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }
}

impl eframe::App for LibretaktUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                if ui.button("T1").clicked() {
                    info!("Quit");
                }

                if ui.button("T2").clicked() {
                    info!("Quit");
                }

                if ui.button("T3").clicked() {
                    info!("Quit");
                }

                if ui.button("T4").clicked() {
                    info!("Quit");
                }

                if ui.button("Mixer").clicked() {
                    info!("Quit");
                }
            });
        });
    }
}
