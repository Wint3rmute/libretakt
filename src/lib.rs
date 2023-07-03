//! Collaborative sampler!

#[cfg(not(target_arch = "wasm32"))]
pub mod constants;
#[cfg(not(target_arch = "wasm32"))]
pub mod engine;
#[cfg(not(target_arch = "wasm32"))]
pub mod persistence;
#[cfg(not(target_arch = "wasm32"))]
pub mod sample_provider;
#[cfg(not(target_arch = "wasm32"))]
pub mod sequencer;

#[cfg(target_arch = "wasm32")]
pub mod ui;
