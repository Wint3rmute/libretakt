use rodio::source::Source;
use rodio::{OutputStreamBuilder, Sink};
use std::time::Duration;

const SAMPLE_RATE: u32 = 48000;

struct SineWaveSource {
    phase: f32,
    phase_diff: f32,
}

impl SineWaveSource {
    fn from_frequency(frequency_hz: f32) -> Self {
        SineWaveSource {
            phase: 0.0,
            phase_diff: (2.0 * std::f32::consts::PI * frequency_hz) / SAMPLE_RATE as f32,
        }
    }
}

impl Iterator for SineWaveSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.phase += self.phase_diff;

        Some(self.phase.sin())
    }
}

impl Source for SineWaveSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() {
    let stream = OutputStreamBuilder::open_default_stream().unwrap();
    let sink = Sink::connect_new(stream.mixer());

    let source = SineWaveSource::from_frequency(440.0);
    sink.append(source);

    sink.sleep_until_end();
}
