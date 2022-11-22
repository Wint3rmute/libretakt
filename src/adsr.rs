pub enum Phase {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

pub struct Adsr {
    phase: Phase,
    state: f32,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            phase: Phase::Finished,
            state: 0.0,
            attack: 0.0001,
            decay: 0.0001,
            sustain: 0.9,
            release: 0.001,
        }
    }
}

impl Adsr {
    pub fn tick(&mut self, note_on: bool) -> f32 {
        if note_on {
            match self.phase {
                Phase::Attack => {
                    self.state += self.attack;
                    if self.state > 1.0 {
                        self.state = 1.0;
                        self.phase = Phase::Decay;
                    }
                }
                Phase::Decay => {
                    self.state -= self.decay;
                    if self.state < self.sustain {
                        self.phase = Phase::Sustain;
                        self.state = self.sustain;
                    }
                }
                Phase::Sustain => {}
                Phase::Release | Phase::Finished => {
                    self.state = 0.0;
                    self.phase = Phase::Attack
                }
            }
        } else {
            match self.phase {
                Phase::Finished => {}
                Phase::Release => {
                    self.state -= self.release;
                    if self.state < 0.0 {
                        self.state = 0.0;
                        self.phase = Phase::Finished;
                    }
                }
                _ => {
                    self.phase = Phase::Release;
                }
            }
        }
        self.state
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
        self.phase = Phase::Attack;
    }
}
