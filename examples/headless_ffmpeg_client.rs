use env_logger::Env;
use flume::{bounded, Receiver};
use libretakt::constants::NUM_OF_VOICES;
use libretakt::engine::{Engine, Voice};
use libretakt::mutation_websocket;
use libretakt::persistence::{load_project, save_project};
use libretakt::sample_provider::SampleProvider;
use libretakt::sequencer::{CurrentStepData, Sequencer, SynchronisationController, Track};
use log::info;
use log::warn;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let tracks = load_project();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him 👉👈.
    let provider = Arc::new(SampleProvider::default());
    let mut synchronisation_controller = Arc::new(Mutex::new(SynchronisationController::default()));

    let (current_step_tx, current_step_rx) = bounded::<CurrentStepData>(64);

    let _voice = Voice::new(&provider);
    let mut engine = Engine {
        sequencer: Sequencer::new(
            synchronisation_controller.lock().unwrap().register_new(),
            current_step_tx.clone(),
            tracks.clone(),
        ),
        voices: (0..NUM_OF_VOICES).map(|_| Voice::new(&provider)).collect(),
    };

    warn!("Synchronisation enabled, connecting to synchronisation server..");
    let mutation_rx_for_sync_server = synchronisation_controller.lock().unwrap().register_new();

    let sync_controller_clone = synchronisation_controller.clone();
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            mutation_websocket::send_mutations_to_server(
                mutation_rx_for_sync_server,
                sync_controller_clone,
            )
            .await
        })
    });

    let mut stdout = io::stdout().lock();

    while let Some(sample) = engine.next() {
        stdout.write_all(&sample.to_le_bytes()).unwrap();
        current_step_rx.try_recv();
    }
}
