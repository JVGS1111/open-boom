//! # OpenBoom Storage
//!
//! Offline-first AutoEq database index and atomic JSON preset & settings persistence.

pub mod autoeq_db;
pub mod preset_store;

pub use autoeq_db::{AutoEqDatabase, AutoEqProfile, ParametricFilterConfig};
pub use preset_store::{AppConfig, PresetStore};
