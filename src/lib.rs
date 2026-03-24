#[cfg(target_arch = "wasm32")]
pub mod agent;
#[cfg(target_arch = "wasm32")]
pub mod app;
pub mod codegen;
#[cfg(target_arch = "wasm32")]
mod date_component;
pub mod disambiguation;
#[cfg(target_arch = "wasm32")]
mod item_component;
#[cfg(target_arch = "wasm32")]
mod platform_component;
pub mod prng;
pub mod traveling_merchant;
pub mod xxhash;
