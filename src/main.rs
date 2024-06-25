use env_logger::Env;

// When compiling the PC/laptop sound engine server
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // use libretakt::persistence::{load_project, save_project};
    use libretakt::sample_provider::SampleProvider;
    use std::sync::{Arc, Mutex};

    //***SAMPLER***
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // let tracks = load_project();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him 👉👈.
    let provider = Arc::new(SampleProvider::default());
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use reqwasm::websocket;
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(libretakt::ui::LibretaktUI::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
