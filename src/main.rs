#[cfg(not(target_arch = "wasm32"))]
mod server;

// When compiling the PC/laptop sound engine server
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    server::main();
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use libretakt::app_state::create_channels;
    use std::time::Duration;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let (app_state, ws_channels) = create_channels();

    let web_options = eframe::WebOptions {
        follow_system_theme: true,
        // default_theme: eframe::Theme::Light,
        ..eframe::WebOptions::default()
    };

    log::info!("Spawning UI thread...");
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(libretakt::ui::LibretaktUI::new(cc, app_state))),
            )
            .await
            .expect("failed to start eframe");
    });

    log::info!("Spawning websocket thread...");
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let options = ewebsock::Options::default();

            ws_channels
                .to_ui
                .unbounded_send("Connecting to websocket...".into())
                .ok();
            let (_sender, receiver) = ewebsock::connect("ws://localhost:3000/ws", options).unwrap();

            while let Some(event) = receiver.try_recv() {
                log::info!("Received {:?}", event);
                // Forward to UI via channel:
                // ws_channels.to_ui.unbounded_send(...).ok();
            }

            gloo_timers::future::sleep(Duration::from_secs(1)).await;
            ws_channels
                .to_ui
                .unbounded_send("Websocket disconnected!".into())
                .ok();
            log::error!("Websocket disconnected! Attempting to reconnect...");
            gloo_timers::future::sleep(Duration::from_secs(5)).await;
        }
    });
}
