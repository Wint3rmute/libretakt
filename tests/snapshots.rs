#[test]
fn hello_world_snapshot() {
    let mut harness = egui_kittest::Harness::new_ui(|ui| {
        ui.label("Hello, world!");
        let _ = ui.button("Click me");
    });

    harness.snapshot("hello_world");
}
