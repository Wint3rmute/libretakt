//! Demonstrates how to use the Sequencer:
//!  - Plugs the sequencer into a dummy audio source, calling Sequencer.tick() during each sapmle generation
//!  - Prints "Time to tick!" each time a tick event occurs
//!
//! Change the value of `beats_per_minute` to observe a different tick rate

use libretakt::constants;
use libretakt::sequencer::Sequencer;
use rodio::source::Source;
use rodio::{OutputStream, Sink};

struct DummySource {
    sequencer: Sequencer,
}

impl Source for DummySource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        constants::SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for DummySource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.sequencer.tick();
        Some(0.0)
    }
}

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let sequencer = Sequencer {
        beats_per_minute: 120,
        ..Default::default()
    };
    let source = DummySource { sequencer };
    sink.append(source);
    println!("Seqencer started");

    sink.sleep_until_end();
}
