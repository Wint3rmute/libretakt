use crate::frontend::notifications::NotificationQueue;

pub fn show_bottom_panel(
    notifications: &mut NotificationQueue,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) {
    let now = ctx.input(|i| i.time);
    if let Some((msg, alpha)) = notifications.current(now) {
        let base = ui.visuals().text_color();
        let color = egui::Color32::from_rgba_unmultiplied(
            base.r(),
            base.g(),
            base.b(),
            (alpha * 255.0) as u8,
        );
        ui.colored_label(color, msg);
        ctx.request_repaint(); // keep animating during fade
    } else {
        ui.label(""); // maintain consistent panel height
    }
}
