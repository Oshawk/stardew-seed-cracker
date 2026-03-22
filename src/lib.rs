// Core logic — always compiled, public so native binaries can use them.
pub mod codegen;
pub mod game_data;
pub mod observation;
pub mod prng;
pub mod quest_checker;

// UI / WASM — only compiled when targeting wasm32.
#[cfg(target_arch = "wasm32")]
pub mod agent;
#[cfg(target_arch = "wasm32")]
pub mod app;
#[cfg(target_arch = "wasm32")]
mod dropdown;
#[cfg(target_arch = "wasm32")]
mod observation_row;
#[cfg(target_arch = "wasm32")]
mod platform_component;
