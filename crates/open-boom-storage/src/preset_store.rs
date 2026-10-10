//! Atomic JSON preset and configuration persistence in %APPDATA%/OpenBoom/.

use open_boom_core::types::{GRAPHIC_EQ_BANDS, ITU_R_7_1_DEFAULT_AZIMUTHS, SURROUND_7_1_CHANNELS};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Top-level OpenBoom user settings and preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
    pub speaker_muted: [bool; SURROUND_7_1_CHANNELS],
    pub speaker_gains_db: [f32; SURROUND_7_1_CHANNELS],
    pub speaker_azimuths_deg: [f32; SURROUND_7_1_CHANNELS],
    pub bypass_active: bool,
    pub launch_on_startup: bool,
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
            surround_enabled: true,
            surround_intensity: 0.72,
            speaker_muted: [false; SURROUND_7_1_CHANNELS],
            speaker_gains_db: [0.0; SURROUND_7_1_CHANNELS],
            speaker_azimuths_deg: ITU_R_7_1_DEFAULT_AZIMUTHS,
            bypass_active: false,
            launch_on_startup: false,
        }
    }
}

/// Storage coordinator managing atomic file operations in `%APPDATA%\OpenBoom\`.
#[derive(Debug, Clone)]
pub struct PresetStore {
    base_dir: PathBuf,
}

impl Default for PresetStore {
    fn default() -> Self {
        Self::new(Self::default_dir())
    }
}

impl PresetStore {
    /// Resolves the default OpenBoom storage directory: `%APPDATA%\OpenBoom\`.
    pub fn default_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("OpenBoom")
        } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
            PathBuf::from(userprofile)
                .join("AppData")
                .join("Roaming")
                .join("OpenBoom")
        } else {
            std::env::temp_dir().join("OpenBoom")
        }
    }

    /// Initializes a `PresetStore` targeting the specified directory.
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Target directory where configurations and presets are stored.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Target path for main `config.json`.
    pub fn config_path(&self) -> PathBuf {
        self.base_dir.join("config.json")
    }

    /// Atomically persists configuration using a unique `.tmp` file and atomic replacement.
    pub fn save_config(&self, config: &AppConfig) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let target_path = self.config_path();
        let tmp_path = self.base_dir.join(format!("config.{}.tmp", std::process::id()));

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write to staging file first
        if let Err(e) = fs::write(&tmp_path, json) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }

        // Atomically replace target file using rename (MoveFileExW MOVEFILE_REPLACE_EXISTING on Windows)
        if let Err(e) = fs::rename(&tmp_path, &target_path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }

        Ok(())
    }

    /// Loads configuration from disk. Recovers defaults gracefully if file is missing or invalid JSON.
    pub fn load_config(&self) -> io::Result<AppConfig> {
        let target_path = self.config_path();
        if !target_path.exists() {
            return Ok(AppConfig::default());
        }

        let content = match fs::read_to_string(&target_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read OpenBoom config at {:?}: {}. Using defaults.",
                    target_path, e
                );
                return Ok(AppConfig::default());
            }
        };

        match serde_json::from_str::<AppConfig>(&content) {
            Ok(cfg) => Ok(cfg),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to parse OpenBoom config.json ({:?}): {}. Falling back to default configuration.",
                    target_path, e
                );
                Ok(AppConfig::default())
            }
        }
    }
}
