pub struct Sequencer {
    pub tracks: Vec<Track>,
}

impl Default for Sequencer {
    fn default() -> Self {
        let mut tracks = vec![];
        tracks.resize_with(1, Track::default);
        Self { tracks }
    }
}

pub struct Track {
    pub steps: Vec<Step>,
    pub current_step_num: usize,
}

impl Default for Track {
    fn default() -> Self {
        let mut steps = vec![];
        steps.resize_with(16, Step::default);
        Self {
            steps,
            current_step_num: 0,
        }
    }
}

pub struct Step {
    pub set: bool,
}

impl Default for Step {
    fn default() -> Self {
        Self { set: false }
    }
}
