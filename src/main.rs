use macroquad::prelude::*;
use rodio::decoder::Decoder;
use rodio::source::Source;
use rodio::{OutputStream, Sink};

use std::sync::{Arc, Mutex};
use std::time::Duration;

static SAMPLE_RATE: usize = 44100;

struct SharedSample<T: rodio::Sample + Source + Iterator>(Arc<Mutex<T>>)
where
    <T as Iterator>::Item: rodio::Sample;

// impl<T: Source + Iterator> SharedSample<T> {
//     pub fn new(source: T) {
//         Arc::new(Mutex::new(source))
//     }
// }

struct Sample {
    data: Vec<f32>,
    play_position: f32,
    playback_speed: Arc<Mutex<f32>>,
}

// impl Iterator for SharedSample {
//     type Item = f32;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.0.lock().unwrap().next()
//     }
// }

impl Iterator for Sample {
    type Item = f32;

    /*
    It should work like this:

    Each sound has a "playback position", from 0.0 to <num_of_samples>.
    When a next sample is requested, it is calculated as follows:
    - Find the 2 samples closest to the playback position
    - Return a weighted average

    Position = 112.2
    Total number of points in the sample: 128

    128 * 0.23 = 29.44
    Distance from:
        - sample 112 => 0.2
        - sample 113 => 0.8

    Result: avg(
        Sample 112 * 0.2
        Sample 113 * 0.8
    )
    */
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.get_at_index(self.play_position);
        self.play_position += *self.playback_speed.lock().unwrap();

        if (self.play_position + 1.0) >= self.data.len() as f32 {
            self.play_position = 0.0;
        }

        return Some(result);
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
        // self.data[left_sample as usize]
    }
}

impl Source for Sample {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[macroquad::main("LibreTakt")]
async fn main() {
    let file = std::fs::File::open("./samples/small_arpeggio.wav").unwrap();

    let mut d = Decoder::new_wav(file).unwrap();
    println!("{}", d.sample_rate());
    println!("{:?}", d.channels());

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _sink = Sink::try_new(&stream_handle).unwrap();

    let mut sample_data: Vec<f32> = vec![];

    while let Some(s) = d.next() {
        sample_data.push(s as f32 / i16::MAX as f32);
        d.next(); // Skip the 2nd channel
    }

    // println!("Enter playback speed");
    let _buffer = String::new();
    // io::stdin().read_line(&mut buffer).unwrap();
    let mut playback_speed = Arc::new(Mutex::new(0.5)); // buffer.trim().parse::<f32>().unwrap();

    let sample = Sample {
        play_position: 0.0,
        playback_speed: playback_speed.clone(),
        data: sample_data.clone(),
    };

    _sink.append(sample);
    _sink.play();
    // sink.sleep_until_end();

    // exit(0);
    // close_current_window();

    let visualisation_size = 400;
    loop {
        clear_background(BLACK);

        // for sample_pos in 0..visualisation_size - 1 {
        //     let val_before = sample.get_at_index(sample_pos as f32 / visualisation_size as f32);
        //     let val_after =
        //         sample.get_at_index((sample_pos + 1) as f32 / visualisation_size as f32);
        //     println!("{} {}", val_before, val_after);

        //     draw_line(
        //         sample_pos as f32,
        //         (val_before as f32 / (i16::MAX as f32 / 100.0)) as f32 + 100.0,
        //         (sample_pos + 1) as f32,
        //         (val_after as f32 / (i16::MAX as f32 / 100.0)) as f32 + 100.0,
        //         1.0,
        //         WHITE,
        //     );
        // }

        let position = mouse_position_local();
        *playback_speed.lock().unwrap() = position.y + 1.0;
        // draw_rectangle(100.0, (position.y + 1.0) * 100.0, 10.0, 10.0, BLUE);
        // draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        // draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

        // draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
