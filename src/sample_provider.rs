//! Reads sample data from files and and provides it to other components.
use std::path::Path;

use log::info;
use rodio::decoder::Decoder;

/// Reads samples from files and provides an interface to access them.
///
/// Should be wrapped inside an [Arc](std::sync::Arc)
/// for read-only access for voices and UI.
pub struct SampleProvider {
    pub samples: Vec<SampleData>,
}

impl Default for SampleProvider {
    fn default() -> Self {
        let paths = std::fs::read_dir("./samples").unwrap();

        let mut samples = vec![];

        for path in paths {
            let path = path.unwrap().path();

            if path.extension().unwrap() != "wav" {
                continue;
            }
            let path_display = path.display();
            info!("Loading {path_display}");

            samples.push(SampleData::from_file(path.as_path()));
        }

        Self { samples }
    }
}

/// Data of a single sample used within Libretakt
pub struct SampleData {
    pub name: String,
    pub data: Vec<f32>,
}

impl SampleData {
    fn from_file(path: &Path) -> Self {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file = std::fs::File::open(path).unwrap();
        let mut decoder = Decoder::new_wav(file).unwrap();

        let mut sample_data: Vec<f32> = vec![];

        while let Some(s) = decoder.next() {
            sample_data.push(s as f32 / i16::MAX as f32);
            decoder.next(); // Skip the 2nd channel
        }

        SampleData {
            data: sample_data,
            name: file_name.to_string(),
        }
    }
}
