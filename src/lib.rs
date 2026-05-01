//! Collaborative sampler!

pub mod shared;

#[cfg(not(target_arch = "wasm32"))]
pub mod constants;
#[cfg(not(target_arch = "wasm32"))]
pub mod engine;
#[cfg(not(target_arch = "wasm32"))]
pub mod sample_provider;

#[cfg(target_arch = "wasm32")]
pub mod frontend;
