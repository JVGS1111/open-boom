//! # OpenBoom Storage
//!
//! Offline-first AutoEq database index and atomic JSON preset & settings persistence.

pub mod autoeq_db;
pub mod preset_store;
pub mod startup;

pub use autoeq_db::{
    AutoEqDatabase, AutoEqProfile, AutoEqProfileSummary, AutoEqStats, ParametricFilterConfig,
};
pub use preset_store::{
    sanitize_audio_bridge_mode, AppConfig, AppVolumeProfile, CoreOrbitDirection as OrbitDirection,
    EqPreset, PresetStore,
};
pub use startup::{StartupManager, DEFAULT_APP_NAME, RUN_KEY_SUBKEY};
