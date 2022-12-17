fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let tracks = load_project();

    //To be honest i haven't been looking at this code yet but Bączek wrote it
    //so i guess its something important and i trust him 👉👈.
    let provider = Arc::new(SampleProvider::default());
    let mut synchronisation_controller = Arc::new(Mutex::new(SynchronisationController::default()));

    let (current_step_tx, current_step_rx) = bounded::<CurrentStepData>(64);

    let _voice = Voice::new(&provider);
    let engine = Engine {
        sequencer: Sequencer::new(
            synchronisation_controller.lock().unwrap().register_new(),
            current_step_tx.clone(),
            tracks.clone(),
        ),
        voices: (0..NUM_OF_VOICES).map(|_| Voice::new(&provider)).collect(),
    };
}
