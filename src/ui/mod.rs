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
        egui::TopBottomPanel::top("track_select").show(ctx, |ui| {
            ui.columns(4, |columns| {
                columns[0].button("T1");
                columns[1].button("T2");
                columns[2].button("T3");
                columns[3].button("T4");
            });
        });
        egui::CentralPanel::default().show(&ctx, |ui| {
            let num_cols = 4;
            let num_rows = 8;

            for _ in 0..num_rows {
                ui.columns(num_cols, |columns| {
                    for col in 0..num_cols {
                        columns[col].add(egui::widgets::Button::new(" \n ").rounding(0.0));
                    }
                });
            }
        });
    }
}
