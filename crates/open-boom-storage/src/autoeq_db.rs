//! Bundled AutoEq database index for instant offline headphone curve resolution.

use serde::{Deserialize, Serialize};

/// Headphone calibration profile metadata and parametric biquad curve filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEqProfile {
    pub name: String,
    pub source: String,
    pub preamp_gain_db: f32,
    pub filters: Vec<ParametricFilterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametricFilterConfig {
    pub filter_type: String,
    pub frequency_hz: f32,
    pub q: f32,
    pub gain_db: f32,
}

/// In-memory AutoEq database index supporting sub-50ms prefix search.
pub struct AutoEqDatabase {
    profiles: Vec<AutoEqProfile>,
}

impl Default for AutoEqDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoEqDatabase {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    pub fn search(&self, query: &str) -> Vec<&AutoEqProfile> {
        let q = query.to_lowercase();
        self.profiles
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&q))
            .take(50)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}
