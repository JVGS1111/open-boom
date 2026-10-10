//! # OpenBoom Storage
//!
//! Offline-first AutoEq database index and atomic JSON preset & settings persistence.

pub mod autoeq_db;
pub mod preset_store;
pub mod startup;

pub use autoeq_db::{AutoEqDatabase, AutoEqProfile, ParametricFilterConfig};
pub use preset_store::{AppConfig, PresetStore};
pub use startup::{StartupManager, DEFAULT_APP_NAME, RUN_KEY_SUBKEY};
