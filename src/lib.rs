//! 三角洲行动同步器: portable product core plus the AstroBox V4 component adapter.
pub mod contracts;
pub mod demo;
pub mod export;
pub mod state;
pub mod store;
pub mod ui;
pub mod view;

#[cfg(target_arch = "wasm32")]
mod runtime;
