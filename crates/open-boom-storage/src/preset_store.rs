//! Atomic JSON preset and configuration persistence in %APPDATA%/OpenBoom/.

use open_boom_core::types::GRAPHIC_EQ_BANDS;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Top-level OpenBoom user settings and preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub master_power: bool,
    pub master_volume: f32,
    pub master_boost: f32,
    pub is_muted: bool,
    pub active_output_device: Option<String>,
    pub window_width: u32,
    pub window_height: u32,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub graphic_eq_gains_db: [f32; GRAPHIC_EQ_BANDS],
    pub surround_enabled: bool,
    pub surround_intensity: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            master_power: true,
            master_volume: 0.8,
            master_boost: 1.0,
            is_muted: false,
            active_output_device: None,
            window_width: 840,
            window_height: 560,
            window_x: None,
            window_y: None,
            graphic_eq_gains_db: [0.0; GRAPHIC_EQ_BANDS],
            surround_enabled: false,
            surround_intensity: 0.7,
        }
    }
}

/// Storage coordinator managing atomic file operations.
pub struct PresetStore {
    base_dir: PathBuf,
}

impl PresetStore {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Atomically persists configuration using a unique .tmp file and atomic rename.
    pub fn save_config(&self, config: &AppConfig) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let target_path = self.base_dir.join("config.json");
        let tmp_path = self.base_dir.join(format!("config.{}.tmp", std::process::id()));

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        fs::write(&tmp_path, json)?;
        fs::rename(&tmp_path, &target_path)?;
        Ok(())
    }

    /// Loads configuration from disk, returning Default if missing.
    pub fn load_config(&self) -> io::Result<AppConfig> {
        let target_path = self.base_dir.join("config.json");
        if !target_path.exists() {
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(&target_path)?;
        serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}
