//! Bundled AutoEq database index for instant offline headphone curve resolution.

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw compressed database bytes bundled at compile time (< 5 MB on disk, ~200 KB gzipped).
const BUNDLED_DB_GZ: &[u8] = include_bytes!("../assets/autoeq_database.bin.gz");

/// Headphone calibration profile metadata and parametric biquad curve filters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoEqProfile {
    pub id: String,
    pub brand: String,
    pub model: String,
    pub name: String,
    pub source: String,
    pub target_curve: String,
    pub preamp_gain_db: f32,
    pub filters: Vec<ParametricFilterConfig>,
}

/// Parametric biquad filter configuration for AutoEq correction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParametricFilterConfig {
    pub filter_type: String, // "Peaking", "LowShelf", "HighShelf"
    pub frequency_hz: f32,
    pub q: f32,
    pub gain_db: f32,
}

/// Lightweight profile summary returned in search results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoEqProfileSummary {
    pub id: String,
    pub brand: String,
    pub model: String,
    pub name: String,
    pub target_curve: String,
    pub preamp_gain_db: f32,
    pub filter_count: usize,
    pub score: i32,
}

/// High-level statistical metadata regarding the bundled AutoEq database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoEqStats {
    pub total_profiles: usize,
    pub total_brands: usize,
    pub database_size_bytes: usize,
}

/// Internal helper for fast search matching
#[derive(Debug, Clone)]
struct SearchEntry {
    index: usize,
    name_clean: String,
    name_raw: String,
    model_clean: String,
    brand_clean: String,
    curve_clean: String,
    tokens: Vec<String>,
}

/// In-memory AutoEq database index supporting sub-50ms fuzzy search and profile lookups.
pub struct AutoEqDatabase {
    profiles: Vec<AutoEqProfile>,
    id_map: HashMap<String, usize>,
    search_entries: Vec<SearchEntry>,
    popular_ids: Vec<String>,
}

impl Default for AutoEqDatabase {
    fn default() -> Self {
        Self::load().unwrap_or_else(|_| Self::new_empty())
    }
}

impl AutoEqDatabase {
    /// Creates an empty database instance (mainly for fallback/testing).
    pub fn new_empty() -> Self {
        Self {
            profiles: Vec::new(),
            id_map: HashMap::new(),
            search_entries: Vec::new(),
            popular_ids: Vec::new(),
        }
    }

    /// Load the bundled pre-compiled compressed database from binary asset.
    pub fn load() -> Result<Self, String> {
        let mut decoder = GzDecoder::new(BUNDLED_DB_GZ);
        let profiles: Vec<AutoEqProfile> = serde_json::from_reader(&mut decoder)
            .map_err(|e| format!("Failed to deserialize bundled AutoEq database: {}", e))?;

        Ok(Self::from_profiles(profiles))
    }

    /// Construct database from a list of profiles, building lookup and search indices.
    pub fn from_profiles(profiles: Vec<AutoEqProfile>) -> Self {
        let mut id_map = HashMap::with_capacity(profiles.len());
        let mut search_entries = Vec::with_capacity(profiles.len());

        for (i, p) in profiles.iter().enumerate() {
            id_map.insert(p.id.clone(), i);

            let name_raw = p.name.to_lowercase();
            let name_clean: String = name_raw.chars().filter(|c| c.is_alphanumeric()).collect();
            let model_clean: String = p.model.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();
            let brand_clean: String = p.brand.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();
            let curve_clean: String = p.target_curve.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect();

            let mut tokens: Vec<String> = name_raw
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            let curve_raw = p.target_curve.to_lowercase();
            for s in curve_raw.split(|c: char| !c.is_alphanumeric()).filter(|s| !s.is_empty()) {
                tokens.push(s.to_string());
            }
            tokens.push(brand_clean.clone());
            tokens.push(model_clean.clone());
            tokens.push(curve_clean.clone());

            search_entries.push(SearchEntry {
                index: i,
                name_clean,
                name_raw,
                model_clean,
                brand_clean,
                curve_clean,
                tokens,
            });
        }

        // Top popular headphone IDs for empty search queries
        let popular_candidates = [
            "hyperx-cloud-ii-oratory1990",
            "sennheiser-hd-600-oratory1990",
            "sony-wh-1000xm5-oratory1990",
            "apple-airpods-max-oratory1990",
            "beyerdynamic-dt-770-pro-80-ohm-oratory1990",
            "audio-technica-ath-m50x-oratory1990",
            "apple-airpods-pro-2-crinacle",
            "sennheiser-hd-560s-oratory1990",
            "sony-wf-1000xm5-crinacle",
            "bose-quietcomfort-45-oratory1990",
            "hyperx-cloud-alpha-oratory1990",
            "sennheiser-hd-650-oratory1990",
            "hifiman-sundara-oratory1990",
            "audeze-maxwell-oratory1990",
            "moondrop-blessing-3-crinacle",
        ];

        let mut popular_ids = Vec::new();
        for &cand in &popular_candidates {
            if id_map.contains_key(cand) {
                popular_ids.push(cand.to_string());
            }
        }

        // If candidates not found, fall back to first 15 profiles
        if popular_ids.is_empty() {
            for p in profiles.iter().take(15) {
                popular_ids.push(p.id.clone());
            }
        }

        Self {
            profiles,
            id_map,
            search_entries,
            popular_ids,
        }
    }

    /// Search headphone profiles with in-memory fuzzy matching and score ranking (< 50ms).
    pub fn search(&self, query: &str) -> Vec<AutoEqProfileSummary> {
        let trimmed = query.trim();

        // 1. Empty query -> return top popular recommendations
        if trimmed.is_empty() {
            return self
                .popular_ids
                .iter()
                .filter_map(|id| self.get_profile(id))
                .map(|p| self.to_summary(p, 100))
                .collect();
        }

        let query_raw = trimmed.to_lowercase();
        let query_clean: String = query_raw.chars().filter(|c| c.is_alphanumeric()).collect();
        let query_tokens: Vec<&str> = query_raw
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        let mut scored_matches: Vec<(usize, i32)> = Vec::new();

        for entry in &self.search_entries {
            let score = compute_score(
                &query_clean,
                &query_raw,
                &query_tokens,
                &entry.name_clean,
                &entry.name_raw,
                &entry.model_clean,
                &entry.brand_clean,
                &entry.curve_clean,
                &entry.tokens,
            );

            if score > 0 {
                scored_matches.push((entry.index, score));
            }
        }

        // Sort descending by score; ties broken by shorter name, then brand
        scored_matches.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| {
                    let pa = &self.profiles[a.0];
                    let pb = &self.profiles[b.0];
                    pa.name.len().cmp(&pb.name.len()).then_with(|| pa.brand.cmp(&pb.brand))
                })
        });

        // Limit results to top 50
        scored_matches
            .into_iter()
            .take(50)
            .map(|(idx, score)| {
                let p = &self.profiles[idx];
                self.to_summary(p, score)
            })
            .collect()
    }

    /// Retrieve all indexed profiles.
    pub fn profiles(&self) -> &[AutoEqProfile] {
        &self.profiles
    }

    /// Retrieve a specific headphone profile by unique ID.
    pub fn get_profile(&self, id: &str) -> Option<&AutoEqProfile> {
        self.id_map.get(id).map(|&idx| &self.profiles[idx])
    }

    /// Retrieve database statistics (total profiles, brands, bundled size).
    pub fn stats(&self) -> AutoEqStats {
        let mut brands = std::collections::HashSet::new();
        for p in &self.profiles {
            brands.insert(&p.brand);
        }

        AutoEqStats {
            total_profiles: self.profiles.len(),
            total_brands: brands.len(),
            database_size_bytes: BUNDLED_DB_GZ.len(),
        }
    }

    /// Raw bundled compressed database size in bytes.
    pub fn bundled_size_bytes() -> usize {
        BUNDLED_DB_GZ.len()
    }

    /// Total number of indexed headphone profiles.
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    /// Whether the database is empty.
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    fn to_summary(&self, p: &AutoEqProfile, score: i32) -> AutoEqProfileSummary {
        AutoEqProfileSummary {
            id: p.id.clone(),
            brand: p.brand.clone(),
            model: p.model.clone(),
            name: p.name.clone(),
            target_curve: p.target_curve.clone(),
            preamp_gain_db: p.preamp_gain_db,
            filter_count: p.filters.len(),
            score,
        }
    }
}

/// Compute fuzzy matching score for a search candidate against query.
fn compute_score(
    query_clean: &str,
    query_raw: &str,
    query_tokens: &[&str],
    name_clean: &str,
    name_raw: &str,
    model_clean: &str,
    brand_clean: &str,
    curve_clean: &str,
    candidate_tokens: &[String],
) -> i32 {
    if query_clean.is_empty() {
        return 0;
    }

    // 1. Exact match on full name (case-insensitive) -> score 100
    if name_clean == query_clean || name_raw == query_raw {
        return 100;
    }

    // 2. Exact match on model -> score 98
    if model_clean == query_clean {
        return 98;
    }

    // 3. Name or clean name starts with query -> score 92
    if name_clean.starts_with(query_clean) || name_raw.starts_with(query_raw) {
        return 92;
    }

    // 4. Model starts with query -> score 88
    if model_clean.starts_with(query_clean) {
        return 88;
    }

    // 5. Multi-token match: all query tokens present in candidate tokens -> score 85
    if query_tokens.len() > 1 {
        let all_match = query_tokens.iter().all(|qt| {
            candidate_tokens.iter().any(|ct| ct.contains(qt))
        });
        if all_match {
            return 85;
        }
    }

    // 6. Brand starts with query -> score 82
    if brand_clean.starts_with(query_clean) {
        return 82;
    }

    // 7. Model contains clean query as substring -> score 80
    if model_clean.contains(query_clean) {
        return 80;
    }

    // 8. Full name contains clean query as substring -> score 75
    if name_clean.contains(query_clean) || name_raw.contains(query_raw) {
        return 75;
    }

    // 9. Target curve contains query -> score 60
    if curve_clean.contains(query_clean) {
        return 60;
    }

    // 10. Subsequence fuzzy match (letters appear in order, min 3 chars) -> score 45
    if query_clean.len() >= 3
        && (is_subsequence(query_clean, name_clean) || is_subsequence(query_clean, model_clean))
    {
        return 45;
    }

    0
}

/// Check if `sub` is a subsequence of `target`.
fn is_subsequence(sub: &str, target: &str) -> bool {
    let mut sub_chars = sub.chars();
    let mut current_sub = match sub_chars.next() {
        Some(c) => c,
        None => return true,
    };

    for target_char in target.chars() {
        if target_char == current_sub {
            match sub_chars.next() {
                Some(next_c) => current_sub = next_c,
                None => return true,
            }
        }
    }

    false
}
