//! Example module-level docstring

use macroquad::prelude::*;
use rodio::decoder::Decoder;
use rodio::source::Source;
use rodio::{OutputStream, Sink};

use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod constants;
pub mod engine;
pub mod sample_provider;
pub mod sequencer;

struct Sample {
    data: Vec<f32>, // TODO: replace with a global sample data provider
    play_position: f32,
    playback_speed: Arc<Mutex<f32>>,
    // TODO: add more playback logic:
    // - Play in reverse
    // - Looping
}
impl Iterator for Sample {
    type Item = f32;

    /// TLDR linear interpolation for sample playback,
    /// allowing for speeding up and slowing down samples:
    ///
    /// Each sound has a "playback position", from 0.0 to <num_of_samples>.
    /// When a next sample is requested, it is calculated as follows:
    /// - Find the 2 samples closest to the playback position
    /// - Return a weighted average
    ///
    /// Example:
    /// Position = 112.2
    /// Total number of points in the sample: 128
    ///
    /// 128 * 0.23 = 29.44
    /// Distance from:
    ///     - sample 112 => 0.2
    ///     - sample 113 => 0.8
    ///
    /// Result: avg(
    ///     Sample 112 * 0.2
    ///     Sample 113 * 0.8
    /// )
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.get_at_index(self.play_position);
        self.play_position += *self.playback_speed.lock().unwrap();

        if (self.play_position + 1.0) >= self.data.len() as f32 {
            self.play_position = 0.0;
        }

        Some(result)
    }
}

impl Sample {
    fn get_at_index(&self, sample_position: f32) -> f32 {
        let left_sample = sample_position.floor();
        let right_sample = left_sample + 1.0;

        let distance_from_left_sample = sample_position - left_sample;
        let distance_from_right_sample = 1.0 - distance_from_left_sample;

        (self.data[left_sample as usize] as f32 * (sample_position - left_sample))
            + (self.data[right_sample as usize] as f32 * distance_from_right_sample)
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
        constants::SAMPLE_RATE as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        None // Again, means that the sound will play indefinitely
    }
}

#[macroquad::main("LibreTakt")]
async fn main() {
    let provider = sample_provider::SampleProvider::default();

    exit(0);

    let file = std::fs::File::open("./samples/small_arpeggio.wav").unwrap();

    let mut d = Decoder::new_wav(file).unwrap();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _sink = Sink::try_new(&stream_handle).unwrap();

    let mut sample_data: Vec<f32> = vec![];

    while let Some(s) = d.next() {
        sample_data.push(s as f32 / i16::MAX as f32);
        d.next(); // Skip the 2nd channel
    }

    let _buffer = String::new();
    let playback_speed = Arc::new(Mutex::new(0.5));

    let sample = Sample {
        play_position: 0.0,
        playback_speed: playback_speed.clone(),
        data: sample_data.clone(),
    };

    _sink.append(sample);
    _sink.play();

    loop {
        clear_background(BLACK);
        let position = mouse_position_local();
        let new_speed = (position.y + 1.0) / 2.0;
        println!("{new_speed}");
        *playback_speed.lock().unwrap() = new_speed;

        next_frame().await
    }
}
