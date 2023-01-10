use crate::constants::NUM_OF_VOICES;
use crate::sequencer::{CurrentStepData, Sequencer, SynchronisationController, Track};
use log::info;
use std::io::Read;

pub fn load_project() -> Vec<Track> {
    let mut project_path = std::env::temp_dir();
    project_path.push("project.json");

    if let Ok(mut file) = std::fs::File::open(project_path) {
        info!("Attempting to load tracks from /tmp/project.json..");
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        let tracks: Vec<Track> = serde_json::from_str(buf.as_str()).unwrap();
        info!("Tracks loaded!");
        tracks
    } else {
        info!("No snapshot file, starting from scratch");

        (0..NUM_OF_VOICES).map(|_| Track::new()).collect()
    }
}

pub fn save_project(sequencer: &Sequencer) {
    use std::io::Write;

    let mut project_path = std::env::temp_dir();
    project_path.push("project.json");

    let serialised = serde_json::to_vec(&sequencer.tracks).unwrap();
    let mut file = std::fs::File::create(project_path).unwrap();
    file.write_all(&serialised).unwrap();
}
