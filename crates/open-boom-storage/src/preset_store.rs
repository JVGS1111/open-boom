//! Atomic JSON preset and configuration persistence in %APPDATA%/OpenBoom/.

use open_boom_core::types::{
    GRAPHIC_EQ_BANDS, ITU_R_7_1_DEFAULT_AZIMUTHS, OrbitDirection, SURROUND_7_1_CHANNELS,
};
pub use open_boom_core::types::{AppVolumeProfile, OrbitDirection as CoreOrbitDirection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn default_active_preset() -> String {
    "Flat".to_string()
}

/// Represents a 31-band Graphic Equalizer preset (either built-in factory or user custom).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EqPreset {
    pub id: String,
    pub name: String,
    pub category: String,
    pub is_factory: bool,
    pub gains_db: [f32; GRAPHIC_EQ_BANDS],
}

impl EqPreset {
    /// Sanitizes all 31 band gains to finite values within [-12.0, +12.0] dB.
    pub fn sanitized(mut self) -> Self {
        for g in &mut self.gains_db {
            if !g.is_finite() {
                *g = 0.0;
            } else {
                *g = g.clamp(-12.0, 12.0);
            }
        }
        self
    }
}

fn default_true() -> bool {
    true
}

fn default_amount_ambience() -> f32 {
    0.35
}

fn default_amount_fidelity() -> f32 {
    0.45
}

fn default_amount_night_mode() -> f32 {
    0.50
}

fn default_amount_spatial() -> f32 {
    0.60
}

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
    #[serde(default)]
    pub autoeq_profile_id: Option<String>,
    #[serde(default = "default_active_preset")]
    pub active_preset_name: String,
    #[serde(default = "default_true")]
    pub sweetener_ambience_enabled: bool,
    #[serde(default = "default_amount_ambience")]
    pub sweetener_ambience_amount: f32,
    #[serde(default = "default_true")]
    pub sweetener_fidelity_enabled: bool,
    #[serde(default = "default_amount_fidelity")]
    pub sweetener_fidelity_amount: f32,
    #[serde(default)]
    pub sweetener_night_mode_enabled: bool,
    #[serde(default = "default_amount_night_mode")]
    pub sweetener_night_mode_amount: f32,
    #[serde(default = "default_true")]
    pub sweetener_spatial_enabled: bool,
    #[serde(default = "default_amount_spatial")]
    pub sweetener_spatial_amount: f32,
    #[serde(default)]
    pub sweetener_pitch_semitones: i32,
    #[serde(default = "default_true")]
    pub buffer_guard_enabled: bool,
    #[serde(default)]
    pub orbit_enabled: bool,
    #[serde(default = "default_orbit_direction")]
    pub orbit_direction: OrbitDirection,
    #[serde(default = "default_orbit_period_sec")]
    pub orbit_period_sec: f32,
    #[serde(default = "default_orbit_depth")]
    pub orbit_depth: f32,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default = "default_true")]
    pub check_updates_on_startup: bool,
    #[serde(default)]
    pub auto_install_updates: bool,
}

fn default_orbit_direction() -> OrbitDirection {
    OrbitDirection::Clockwise
}

fn default_orbit_period_sec() -> f32 {
    12.0
}

fn default_orbit_depth() -> f32 {
    0.85
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            master_power: true,
            master_volume: 0.8,
            master_boost: 1.0,
            is_muted: false,
            active_output_device: None,
            window_width: 1000,
            window_height: 620,
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
            autoeq_profile_id: None,
            active_preset_name: default_active_preset(),
            sweetener_ambience_enabled: true,
            sweetener_ambience_amount: 0.35,
            sweetener_fidelity_enabled: true,
            sweetener_fidelity_amount: 0.45,
            sweetener_night_mode_enabled: false,
            sweetener_night_mode_amount: 0.50,
            sweetener_spatial_enabled: true,
            sweetener_spatial_amount: 0.60,
            sweetener_pitch_semitones: 0,
            buffer_guard_enabled: true,
            orbit_enabled: false,
            orbit_direction: OrbitDirection::Clockwise,
            orbit_period_sec: 12.0,
            orbit_depth: 0.85,
            language: None,
            check_updates_on_startup: true,
            auto_install_updates: false,
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

    /// Target directory for custom equalizer preset `.json` files: `%APPDATA%\OpenBoom\presets\`.
    pub fn presets_dir(&self) -> PathBuf {
        self.base_dir.join("presets")
    }

    /// Returns the 15 calibrated 31-band ISO 1/3-octave factory presets (FR11).
    /// Band order: 20, 25, 31.5, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400, 500,
    /// 630, 800, 1k, 1.25k, 1.6k, 2k, 2.5k, 3.15k, 4k, 5k, 6.3k, 8k, 10k, 12.5k, 16k, 20k Hz.
    pub fn factory_presets() -> Vec<EqPreset> {
        vec![
            EqPreset {
                id: "flat".to_string(),
                name: "Flat".to_string(),
                category: "Studio".to_string(),
                is_factory: true,
                gains_db: [0.0; GRAPHIC_EQ_BANDS],
            },
            EqPreset {
                id: "bass-boost".to_string(),
                name: "Bass Boost".to_string(),
                category: "Signature".to_string(),
                is_factory: true,
                gains_db: [
                    6.0, 6.0, 5.5, 5.0, 4.5, 4.0, 3.5, 2.5, 1.8, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                    0.0,
                ],
            },
            EqPreset {
                id: "treble-boost".to_string(),
                name: "Treble Boost".to_string(),
                category: "Signature".to_string(),
                is_factory: true,
                gains_db: [
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                    0.0, 0.0, 0.0, 0.5, 1.0, 1.5, 2.2, 3.0, 3.8, 4.5, 5.0, 5.5, 5.8, 6.0, 5.5,
                    4.5,
                ],
            },
            EqPreset {
                id: "acoustic".to_string(),
                name: "Acoustic".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    3.5, 3.5, 3.2, 2.8, 2.2, 1.5, 1.0, 0.8, 0.5, 0.2, 0.0, -0.5, -0.5, 0.0, 0.5,
                    1.0, 1.5, 2.0, 2.2, 2.5, 2.5, 2.0, 1.8, 2.0, 2.5, 3.0, 3.2, 3.0, 2.5, 2.0,
                    1.5,
                ],
            },
            EqPreset {
                id: "classical".to_string(),
                name: "Classical".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    3.5, 3.2, 2.8, 2.4, 1.8, 1.2, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.3, -0.5,
                    -0.5, -0.3, 0.0, 0.3, 0.8, 1.2, 1.6, 2.0, 2.4, 2.8, 3.0, 3.2, 3.2, 3.0, 2.5,
                    2.0,
                ],
            },
            EqPreset {
                id: "electronic".to_string(),
                name: "Electronic".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    5.0, 5.5, 5.2, 4.5, 3.8, 2.8, 1.5, 0.5, -0.5, -1.2, -1.5, -1.2, -0.8, -0.5,
                    0.0, 0.5, 0.8, 1.0, 0.8, 0.5, 1.0, 1.8, 2.5, 3.2, 3.8, 4.2, 4.5, 4.5, 4.0,
                    3.5, 2.5,
                ],
            },
            EqPreset {
                id: "gaming".to_string(),
                name: "Gaming".to_string(),
                category: "Media".to_string(),
                is_factory: true,
                gains_db: [
                    2.0, 2.5, 3.0, 2.8, 2.2, 1.5, 0.5, -0.5, -1.2, -1.8, -2.0, -1.5, -0.8, 0.0,
                    0.8, 1.5, 2.2, 2.8, 3.5, 4.2, 4.5, 4.2, 3.8, 3.2, 2.5, 2.0, 1.8, 1.5, 1.0,
                    0.5, 0.0,
                ],
            },
            EqPreset {
                id: "hip-hop".to_string(),
                name: "Hip-Hop".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    5.5, 6.0, 5.8, 5.2, 4.5, 3.8, 2.8, 1.8, 0.8, 0.0, -0.8, -1.2, -1.5, -1.2,
                    -0.8, -0.3, 0.2, 0.8, 1.2, 1.5, 1.5, 1.2, 1.0, 1.2, 1.8, 2.2, 2.8, 3.0, 2.8,
                    2.2, 1.5,
                ],
            },
            EqPreset {
                id: "jazz".to_string(),
                name: "Jazz".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    3.2, 3.5, 3.2, 2.8, 2.4, 2.0, 1.5, 1.0, 0.5, 0.2, 0.0, -0.5, -0.8, -0.8,
                    -0.5, 0.0, 0.5, 1.0, 1.4, 1.8, 2.0, 2.0, 1.8, 1.8, 2.0, 2.4, 2.8, 2.8, 2.5,
                    2.0, 1.5,
                ],
            },
            EqPreset {
                id: "movie".to_string(),
                name: "Movie".to_string(),
                category: "Media".to_string(),
                is_factory: true,
                gains_db: [
                    5.0, 5.5, 5.0, 4.2, 3.5, 2.5, 1.5, 0.5, -0.5, -1.0, -1.2, -1.0, -0.5, 0.2,
                    1.0, 1.8, 2.5, 3.0, 3.2, 3.0, 2.5, 2.0, 1.8, 2.0, 2.5, 3.0, 3.2, 3.0, 2.5,
                    2.0, 1.5,
                ],
            },
            EqPreset {
                id: "pop".to_string(),
                name: "Pop".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    2.0, 2.5, 2.8, 3.0, 2.8, 2.2, 1.5, 0.8, 0.2, -0.2, -0.5, -0.5, 0.0, 0.8,
                    1.5, 2.2, 2.8, 3.0, 2.8, 2.5, 2.0, 1.8, 2.0, 2.2, 2.5, 2.8, 3.0, 2.8, 2.2,
                    1.8, 1.2,
                ],
            },
            EqPreset {
                id: "rock".to_string(),
                name: "Rock".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    4.0, 4.5, 4.5, 4.2, 3.8, 3.2, 2.5, 1.5, 0.5, -0.5, -1.2, -1.8, -1.8, -1.2,
                    -0.5, 0.5, 1.5, 2.2, 2.8, 3.2, 3.5, 3.8, 4.0, 4.0, 3.8, 3.5, 3.5, 3.2, 2.8,
                    2.2, 1.5,
                ],
            },
            EqPreset {
                id: "vocal".to_string(),
                name: "Vocal".to_string(),
                category: "Studio".to_string(),
                is_factory: true,
                gains_db: [
                    -3.0, -2.8, -2.5, -2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.2, 1.8, 2.5, 3.0, 3.5,
                    3.8, 4.0, 4.2, 4.2, 4.0, 3.8, 3.5, 3.0, 2.5, 1.8, 1.0, 0.5, 0.0, -0.5, -1.0,
                    -1.5, -2.0,
                ],
            },
            EqPreset {
                id: "lounge".to_string(),
                name: "Lounge".to_string(),
                category: "Music".to_string(),
                is_factory: true,
                gains_db: [
                    3.0, 3.5, 3.8, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0, 0.5, 0.2, 0.0, -0.3, -0.8,
                    -1.2, -1.5, -1.5, -1.2, -0.8, -0.5, 0.0, 0.5, 1.0, 1.5, 1.8, 2.0, 2.2, 2.0,
                    1.8, 1.5, 1.0,
                ],
            },
            EqPreset {
                id: "deep".to_string(),
                name: "Deep".to_string(),
                category: "Signature".to_string(),
                is_factory: true,
                gains_db: [
                    5.0, 5.5, 5.5, 5.0, 4.5, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0, -0.5,
                    -1.0, -1.2, -1.5, -1.5, -1.2, -1.0, -0.8, -0.5, -0.5, -0.8, -1.0, -1.2,
                    -1.5, -1.8, -2.0, -2.5,
                ],
            },
        ]
    }

    /// Sanitizes an arbitrary preset name or identifier into a safe filesystem slug
    /// consisting strictly of lowercase ASCII alphanumeric characters and single hyphens.
    /// Rejects empty slugs and Windows reserved device names.
    pub fn sanitize_preset_slug(input: &str) -> io::Result<String> {
        let mut slug = String::with_capacity(input.len());
        let mut prev_hyphen = true; // Prevents leading hyphen

        for ch in input.trim().chars() {
            if ch.is_ascii_alphanumeric() {
                slug.push(ch.to_ascii_lowercase());
                prev_hyphen = false;
            } else if !prev_hyphen {
                slug.push('-');
                prev_hyphen = true;
            }
        }

        while slug.ends_with('-') {
            slug.pop();
        }

        if slug.len() > 64 {
            slug.truncate(64);
            while slug.ends_with('-') {
                slug.pop();
            }
        }

        if slug.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Preset name must contain at least one ASCII alphanumeric character",
            ));
        }

        const WINDOWS_RESERVED: &[&str] = &[
            "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
            "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8",
            "lpt9",
        ];
        if WINDOWS_RESERVED.contains(&slug.as_str()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Preset name '{}' is a reserved system name", input),
            ));
        }

        Ok(slug)
    }

    /// Checks whether a given name or ID collides with any of the 15 built-in factory presets.
    pub fn is_factory_preset_identifier(name_or_id: &str) -> bool {
        let trimmed = name_or_id.trim();
        if trimmed.is_empty() {
            return false;
        }
        let candidate_slug = Self::sanitize_preset_slug(trimmed).ok();
        Self::factory_presets().iter().any(|fp| {
            fp.name.eq_ignore_ascii_case(trimmed)
                || fp.id.eq_ignore_ascii_case(trimmed)
                || candidate_slug
                    .as_deref()
                    .map(|s| s == fp.id)
                    .unwrap_or(false)
        })
    }

    /// Lists all 15 factory presets followed by all valid user custom presets from disk.
    pub fn list_presets(&self) -> Vec<EqPreset> {
        let mut all = Self::factory_presets();
        all.extend(self.list_custom_presets());
        all
    }

    /// Scans `%APPDATA%\OpenBoom\presets\*.json` and returns all valid user custom presets.
    /// Malformed or corrupt JSON files are logged and skipped without crashing.
    pub fn list_custom_presets(&self) -> Vec<EqPreset> {
        let dir = self.presets_dir();
        if !dir.exists() {
            return Vec::new();
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read OpenBoom presets directory {:?}: {}",
                    dir, e
                );
                return Vec::new();
            }
        };

        let mut custom = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !ext.eq_ignore_ascii_case("json") {
                continue;
            }

            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };

            let slug = match Self::sanitize_preset_slug(stem) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Never allow a disk file to shadow a built-in factory preset
            if Self::is_factory_preset_identifier(&slug) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: Failed to read preset file {:?}: {}", path, e);
                    continue;
                }
            };

            match serde_json::from_str::<EqPreset>(&content) {
                Ok(mut preset) => {
                    if Self::is_factory_preset_identifier(&preset.name) {
                        eprintln!(
                            "Warning: Skipping custom preset {:?} that collides with factory preset name '{}'",
                            path, preset.name
                        );
                        continue;
                    }
                    preset.name = preset.name.trim().to_string();
                    if preset.name.is_empty() {
                        preset.name = slug.clone();
                    }
                    preset.id = slug;
                    preset.is_factory = false;
                    if preset.category.trim().is_empty() {
                        preset.category = "Custom".to_string();
                    }
                    custom.push(preset.sanitized());
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Skipping corrupt preset JSON at {:?}: {}",
                        path, e
                    );
                }
            }
        }

        custom.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
        custom
    }

    /// Loads a single custom preset by ID or name from `%APPDATA%\OpenBoom\presets\<slug>.json`.
    pub fn load_custom_preset(&self, id_or_name: &str) -> io::Result<EqPreset> {
        let slug = Self::sanitize_preset_slug(id_or_name)?;
        if Self::is_factory_preset_identifier(&slug) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("'{}' is a built-in factory preset, not a custom preset", slug),
            ));
        }
        let path = self.presets_dir().join(format!("{}.json", slug));
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Custom preset '{}' not found", slug),
            ));
        }

        let content = fs::read_to_string(&path)?;
        let mut preset: EqPreset = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        preset.name = preset.name.trim().to_string();
        if preset.name.is_empty() {
            preset.name = slug.clone();
        }
        if preset.category.trim().is_empty() {
            preset.category = "Custom".to_string();
        }
        preset.id = slug;
        preset.is_factory = false;
        Ok(preset.sanitized())
    }

    /// Atomically saves a user custom preset to `%APPDATA%\OpenBoom\presets\<slug>.json`
    /// using a temporary staging file and `fs::rename` (`MoveFileExW` on Windows) (FR11, AD-10).
    pub fn save_custom_preset(
        &self,
        name: &str,
        gains_db: [f32; GRAPHIC_EQ_BANDS],
    ) -> io::Result<EqPreset> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Preset name cannot be empty",
            ));
        }

        if Self::is_factory_preset_identifier(trimmed_name) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Cannot overwrite built-in factory preset '{}'",
                    trimmed_name
                ),
            ));
        }

        let slug = Self::sanitize_preset_slug(trimmed_name)?;
        let presets_dir = self.presets_dir();
        fs::create_dir_all(&presets_dir)?;

        let preset = EqPreset {
            id: slug.clone(),
            name: trimmed_name.to_string(),
            category: "Custom".to_string(),
            is_factory: false,
            gains_db,
        }
        .sanitized();

        let target_path = presets_dir.join(format!("{}.json", slug));
        let seq = TMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp_path = presets_dir.join(format!("{}.{}.{}.tmp", slug, std::process::id(), seq));

        let json = serde_json::to_string_pretty(&preset)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Err(e) = fs::write(&tmp_path, json) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }

        if let Err(e) = fs::rename(&tmp_path, &target_path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }

        Ok(preset)
    }

    /// Atomically renames an existing user custom preset, writing the new file first and
    /// removing the old file if the slug changed. Rejects factory presets.
    pub fn rename_custom_preset(&self, old_id_or_name: &str, new_name: &str) -> io::Result<EqPreset> {
        let trimmed_old = old_id_or_name.trim();
        let trimmed_new = new_name.trim();
        if trimmed_old.is_empty() || trimmed_new.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Preset name cannot be empty",
            ));
        }

        if Self::is_factory_preset_identifier(trimmed_old) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Cannot rename built-in factory preset '{}'", trimmed_old),
            ));
        }

        if Self::is_factory_preset_identifier(trimmed_new) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Cannot rename custom preset to factory preset name '{}'",
                    trimmed_new
                ),
            ));
        }

        let old_slug = Self::sanitize_preset_slug(trimmed_old)?;
        let new_slug = Self::sanitize_preset_slug(trimmed_new)?;

        let existing = self.load_custom_preset(&old_slug)?;
        let new_path = self.presets_dir().join(format!("{}.json", new_slug));

        if old_slug != new_slug && new_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("A custom preset named '{}' already exists", trimmed_new),
            ));
        }

        let updated = self.save_custom_preset(trimmed_new, existing.gains_db)?;

        if old_slug != new_slug {
            let old_path = self.presets_dir().join(format!("{}.json", old_slug));
            let _ = fs::remove_file(old_path);
        }

        Ok(updated)
    }

    /// Deletes a user custom preset from `%APPDATA%\OpenBoom\presets\<slug>.json`.
    /// Rejects deleting built-in factory presets.
    pub fn delete_custom_preset(&self, id_or_name: &str) -> io::Result<()> {
        let trimmed = id_or_name.trim();
        if trimmed.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Preset identifier cannot be empty",
            ));
        }

        if Self::is_factory_preset_identifier(trimmed) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Cannot delete built-in factory preset '{}'", trimmed),
            ));
        }

        let slug = Self::sanitize_preset_slug(trimmed)?;
        let target_path = self.presets_dir().join(format!("{}.json", slug));
        if !target_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Custom preset '{}' not found", trimmed),
            ));
        }

        fs::remove_file(target_path)
    }

    /// Atomically persists configuration using a unique `.tmp` file and atomic replacement.
    pub fn save_config(&self, config: &AppConfig) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let target_path = self.config_path();
        let seq = TMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp_path = self
            .base_dir
            .join(format!("config.{}.{}.tmp", std::process::id(), seq));

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
            Ok(mut cfg) => {
                // Defensive sanitization of orbital configuration bounds
                if cfg.orbit_period_sec.is_nan()
                    || cfg.orbit_period_sec < 2.0
                    || cfg.orbit_period_sec > 30.0
                {
                    cfg.orbit_period_sec = 12.0;
                }
                if cfg.orbit_depth.is_nan() || cfg.orbit_depth < 0.0 || cfg.orbit_depth > 1.0 {
                    cfg.orbit_depth = 0.85;
                }
                Ok(cfg)
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to parse OpenBoom config.json ({:?}): {}. Falling back to default configuration.",
                    target_path, e
                );
                Ok(AppConfig::default())
            }
        }
    }

    /// Target path for per-application volume preferences: `%APPDATA%\OpenBoom\app_volumes.json`.
    pub fn app_volumes_path(&self) -> PathBuf {
        self.base_dir.join("app_volumes.json")
    }

    /// Loads per-application volume profiles from `%APPDATA%\OpenBoom\app_volumes.json`.
    /// Returns a map keyed by lowercase executable names for case-insensitive lookup.
    /// Recovers gracefully with an empty map if file is missing or corrupted (AD-10).
    pub fn load_app_volumes(&self) -> io::Result<HashMap<String, AppVolumeProfile>> {
        let target_path = self.app_volumes_path();
        if !target_path.exists() {
            return Ok(HashMap::new());
        }

        let content = match fs::read_to_string(&target_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to read OpenBoom app_volumes.json at {:?}: {}. Using empty profiles.",
                    target_path, e
                );
                return Ok(HashMap::new());
            }
        };

        match serde_json::from_str::<HashMap<String, AppVolumeProfile>>(&content) {
            Ok(map) => {
                let mut normalized = HashMap::with_capacity(map.len());
                for (k, v) in map {
                    normalized.insert(k.to_lowercase(), v);
                }
                Ok(normalized)
            }
            Err(_) => {
                // Fallback attempt: if stored as an array of profiles
                if let Ok(list) = serde_json::from_str::<Vec<AppVolumeProfile>>(&content) {
                    let mut map = HashMap::with_capacity(list.len());
                    for item in list {
                        map.insert(item.process_name.to_lowercase(), item);
                    }
                    Ok(map)
                } else {
                    eprintln!(
                        "Warning: Failed to parse OpenBoom app_volumes.json ({:?}). Falling back to empty profiles.",
                        target_path
                    );
                    Ok(HashMap::new())
                }
            }
        }
    }

    /// Loads per-application volume profiles as a list sorted by process name.
    pub fn load_app_volume_list(&self) -> io::Result<Vec<AppVolumeProfile>> {
        let map = self.load_app_volumes()?;
        let mut list: Vec<AppVolumeProfile> = map.into_values().collect();
        list.sort_by(|a, b| a.process_name.to_lowercase().cmp(&b.process_name.to_lowercase()));
        Ok(list)
    }

    /// Atomically persists per-application volume profiles to `%APPDATA%\OpenBoom\app_volumes.json`
    /// using a temporary staging file and `fs::rename` (MoveFileExW on Windows) (FR23, AD-10).
    pub fn save_app_volumes(&self, profiles: &HashMap<String, AppVolumeProfile>) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let target_path = self.app_volumes_path();
        let seq = TMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp_path = self
            .base_dir
            .join(format!("app_volumes.{}.{}.tmp", std::process::id(), seq));

        let json = serde_json::to_string_pretty(profiles)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if let Err(e) = fs::write(&tmp_path, json) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }

        if let Err(e) = fs::rename(&tmp_path, &target_path) {
            let _ = fs::remove_file(&tmp_path);
            return Err(e);
        }

        Ok(())
    }

    /// Saves or updates a single application volume profile in `%APPDATA%\OpenBoom\app_volumes.json`.
    pub fn save_app_volume_profile(&self, profile: &AppVolumeProfile) -> io::Result<()> {
        let trimmed_name = profile.process_name.trim();
        if trimmed_name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Process name cannot be empty",
            ));
        }

        let mut profiles = self.load_app_volumes()?;
        profiles.insert(
            trimmed_name.to_lowercase(),
            AppVolumeProfile {
                process_name: trimmed_name.to_string(),
                volume: profile.volume.clamp(0.0, 1.5),
                is_muted: profile.is_muted,
            },
        );
        self.save_app_volumes(&profiles)
    }

    /// Retrieves the volume profile for a specific application executable if stored.
    pub fn get_app_volume_profile(&self, process_name: &str) -> io::Result<Option<AppVolumeProfile>> {
        let trimmed = process_name.trim().to_lowercase();
        let profiles = self.load_app_volumes()?;
        Ok(profiles.get(&trimmed).cloned())
    }

    /// Deletes a remembered volume profile for an application. Returns true if profile was present.
    pub fn delete_app_volume_profile(&self, process_name: &str) -> io::Result<bool> {
        let trimmed = process_name.trim().to_lowercase();
        let mut profiles = self.load_app_volumes()?;
        if profiles.remove(&trimmed).is_some() {
            self.save_app_volumes(&profiles)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
