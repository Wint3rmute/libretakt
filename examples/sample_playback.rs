use macroquad::prelude::*;
use rodio::decoder::Decoder;
use rodio::source::Source;
use rodio::{OutputStream, Sink};

use std::time::Duration;

static SAMPLE_RATE: usize = 44100;

struct Sample {
    data: Vec<f32>,
    play_position: usize,
}

impl Sample {
    fn from_file() -> Self {
        let file = std::fs::File::open("./samples/small_arpeggio.wav").unwrap();
        let mut decoder = Decoder::new_wav(file).unwrap();

        let mut sample_data: Vec<f32> = vec![];

        while let Some(s) = decoder.next() {
            sample_data.push(s as f32 / i16::MAX as f32);
            decoder.next(); // Skip the 2nd channel
        }

        Sample {
            play_position: 0,
            data: sample_data,
        }
    }
}

impl Iterator for Sample {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.data[self.play_position];
        self.play_position += 1;

        if self.play_position >= self.data.len() {
            self.play_position = 0;
        }

        Some(result)
    }
}

impl Source for Sample {
    fn current_frame_len(&self) -> Option<usize> {
        None // Means that the sound will play indefinitely
    }

    fn channels(&self) -> u16 {
        1 // We should do 2 in the future :)
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        None // Again, means that the sound will play indefinitely
    }
}

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let source = Sample::from_file();
    sink.append(source);

    sink.sleep_until_end();
}
