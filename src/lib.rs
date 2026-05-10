//! Collaborative sampler!

pub mod shared;

#[cfg(not(target_arch = "wasm32"))]
pub mod constants;
#[cfg(not(target_arch = "wasm32"))]
pub mod engine;
#[cfg(not(target_arch = "wasm32"))]
pub mod sample_provider;

pub mod frontend;
