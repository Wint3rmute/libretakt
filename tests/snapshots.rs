#[test]
fn hello_world_snapshot() {
    let mut harness = egui_kittest::Harness::new_ui(|ui| {
        ui.label("Hello, world!");
        let _ = ui.button("Click me");
    });

    harness.snapshot("hello_world");
}

#[test]
fn app_default_view_snapshot() {
    use libretakt::frontend::{app_state::create_channels, LibretaktUI};

    let (app_state, _ws_channels) = create_channels();
    let app = LibretaktUI::new_for_test(app_state);

    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(390.0, 844.0)) // portrait phone viewport
        .build_ui_state(
            |ui: &mut egui::Ui, libretakt_ui: &mut LibretaktUI| {
                libretakt_ui.render(ui.ctx());
            },
            app,
        );

    harness.snapshot("app_default_view");
}
