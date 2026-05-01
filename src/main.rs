#[cfg(not(target_arch = "wasm32"))]
mod server;

// When compiling the PC/laptop sound engine server
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // use libretakt::persistence::{load_project, save_project};

    server::main();

    //***SAMPLER***
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // let tracks = load_project();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him 👉👈.
    // let provider = Arc::new(SampleProvider::default());
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
            let (_sender, receiver) = ewebsock::connect("ws://localhost:3000", options).unwrap();

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
