//! Unit and integration tests for AutoEq compressed database, index size, 4,000+ count, and sub-50ms fuzzy search.

use open_boom_storage::autoeq_db::{AutoEqDatabase, AutoEqProfile, AutoEqStats};
use std::time::Instant;

#[test]
fn test_database_loads_bundled_asset_with_4000_plus_profiles() {
    let db = AutoEqDatabase::load().expect("AutoEq database must load bundled asset");

    assert!(
        db.len() >= 4000,
        "AutoEq database must contain 4,000+ profiles, actual: {}",
        db.len()
    );
    assert!(!db.is_empty(), "Database must not be empty");

    let stats: AutoEqStats = db.stats();
    assert_eq!(stats.total_profiles, db.len());
    assert!(
        stats.total_brands >= 30,
        "Database should index at least 30 brands, actual: {}",
        stats.total_brands
    );
}

#[test]
fn test_database_compressed_size_is_under_5_mb() {
    let bundled_bytes = AutoEqDatabase::bundled_size_bytes();
    let max_allowed_bytes = 5 * 1024 * 1024; // 5 MB (FR13, AD-9)

    assert!(
        bundled_bytes < max_allowed_bytes,
        "Bundled database on disk must be < 5 MB, actual: {} bytes ({:.2} MB)",
        bundled_bytes,
        bundled_bytes as f64 / (1024.0 * 1024.0)
    );
    // In our implementation it should be < 1 MB (~200 KB)
    assert!(
        bundled_bytes < 1024 * 1024,
        "Bundled database size should ideally be under 1 MB, actual: {} bytes",
        bundled_bytes
    );
}

#[test]
fn test_empty_query_returns_top_popular_recommendations() {
    let db = AutoEqDatabase::load().expect("load database");

    let results = db.search("");
    assert!(
        !results.is_empty(),
        "Empty query should return top recommendations"
    );
    assert!(
        results.len() >= 4,
        "Should return at least 4 popular models"
    );

    let brands: Vec<&str> = results.iter().map(|r| r.brand.as_str()).collect();
    assert!(
        brands.contains(&"HyperX"),
        "Recommendations should include HyperX"
    );
    assert!(
        brands.contains(&"Sennheiser"),
        "Recommendations should include Sennheiser"
    );
    assert!(
        brands.contains(&"Sony"),
        "Recommendations should include Sony"
    );
    assert!(
        brands.contains(&"Apple"),
        "Recommendations should include Apple"
    );
}

#[test]
fn test_exact_brand_and_model_search() {
    let db = AutoEqDatabase::load().expect("load database");

    let start = Instant::now();
    let results = db.search("HyperX Cloud II");
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Search latency must be < 50ms, actual: {:?}",
        elapsed
    );
    assert!(
        !results.is_empty(),
        "Query 'HyperX Cloud II' must return matches"
    );

    let top = &results[0];
    assert_eq!(top.brand, "HyperX");
    assert!(top.model.contains("Cloud II"));
    assert_eq!(
        top.score, 100,
        "Exact match should score highest (100), actual: {}",
        top.score
    );
    assert!(
        top.filter_count >= 5,
        "Profile should have at least 5 parametric filters"
    );
    assert!(
        top.preamp_gain_db <= 0.0,
        "Preamp gain must be negative attenuation offset"
    );
}

#[test]
fn test_partial_and_fuzzy_queries() {
    let db = AutoEqDatabase::load().expect("load database");

    // 1. Query "hd600" -> resolves Sennheiser HD 600
    let hd600_res = db.search("hd600");
    assert!(!hd600_res.is_empty(), "Query 'hd600' must return results");
    let top_hd600 = &hd600_res[0];
    assert_eq!(top_hd600.brand, "Sennheiser");
    assert!(top_hd600.model.contains("HD 600"));
    assert!(top_hd600.score >= 80);

    // 2. Query "xm5" -> resolves Sony WH-1000XM5 or WF-1000XM5
    let xm5_res = db.search("xm5");
    assert!(!xm5_res.is_empty(), "Query 'xm5' must return results");
    let top_xm5 = &xm5_res[0];
    assert_eq!(top_xm5.brand, "Sony");
    assert!(top_xm5.model.contains("1000XM5"));

    // 3. Query "airpods" -> resolves Apple AirPods Max / Pro
    let airpods_res = db.search("airpods");
    assert!(!airpods_res.is_empty(), "Query 'airpods' must return results");
    assert_eq!(airpods_res[0].brand, "Apple");
    assert!(airpods_res[0].model.to_lowercase().contains("airpods"));
}

#[test]
fn test_non_existent_query_returns_empty_results() {
    let db = AutoEqDatabase::load().expect("load database");

    let results = db.search("XYZ123NonExistentHeadphone");
    assert!(
        results.is_empty(),
        "Non-existent query must return empty array, actual: {:?}",
        results
    );
}

#[test]
fn test_sub_50ms_fuzzy_query_latency_benchmark() {
    let db = AutoEqDatabase::load().expect("load database");

    let benchmark_queries = [
        "HyperX Cloud II",
        "hd600",
        "xm5",
        "airpods",
        "dt 770",
        "m50x",
        "sundara",
        "maxwell",
        "crinacle",
        "oratory1990",
        "bose",
        "apple",
        "sony",
        "sennheiser",
        "shure 840",
    ];

    let start_all = Instant::now();
    for &q in &benchmark_queries {
        let single_start = Instant::now();
        let res = db.search(q);
        let single_elapsed = single_start.elapsed();

        assert!(
            single_elapsed.as_millis() < 50,
            "Query '{}' took {:?}, exceeding 50ms latency ceiling (FR14)",
            q,
            single_elapsed
        );
        assert!(!res.is_empty(), "Expected results for query '{}'", q);
    }
    let total_elapsed = start_all.elapsed();
    let avg_ms = total_elapsed.as_secs_f64() * 1000.0 / benchmark_queries.len() as f64;
    println!(
        "Fuzzy search benchmark: {} queries executed in {:?}, avg {:.3} ms/query",
        benchmark_queries.len(),
        total_elapsed,
        avg_ms
    );
    assert!(
        avg_ms < 10.0,
        "Average query time should be well below 10ms, actual: {:.2}ms",
        avg_ms
    );
}

#[test]
fn test_profile_lookup_and_filter_parsing() {
    let db = AutoEqDatabase::load().expect("load database");

    // Lookup HyperX Cloud II by ID
    let profile: &AutoEqProfile = db
        .get_profile("hyperx-cloud-ii-oratory1990")
        .expect("HyperX Cloud II profile must exist");

    assert_eq!(profile.brand, "HyperX");
    assert_eq!(profile.model, "Cloud II");
    assert!(profile.target_curve.contains("oratory1990"));
    assert!(profile.preamp_gain_db <= 0.0);
    assert!(profile.filters.len() >= 5);

    for f in &profile.filters {
        assert!(
            f.filter_type == "Peaking" || f.filter_type == "LowShelf" || f.filter_type == "HighShelf",
            "Filter type must be standard biquad, actual: {}",
            f.filter_type
        );
        assert!(
            f.frequency_hz >= 20.0 && f.frequency_hz <= 20000.0,
            "Filter frequency must be within audible spectrum [20, 20k] Hz, actual: {}",
            f.frequency_hz
        );
        assert!(
            f.q > 0.1 && f.q < 20.0,
            "Filter Q must be positive and bounded, actual: {}",
            f.q
        );
        assert!(
            f.gain_db >= -24.0 && f.gain_db <= 24.0,
            "Filter gain must be within sane parametric bounds, actual: {}",
            f.gain_db
        );
    }
}

#[test]
fn test_all_profile_ids_are_unique_in_database() {
    let db = AutoEqDatabase::load().expect("load database");
    let mut seen_ids = std::collections::HashSet::new();

    for p in db.profiles() {
        assert!(
            seen_ids.insert(p.id.clone()),
            "Duplicate profile ID detected in database: {}",
            p.id
        );
    }
    assert_eq!(seen_ids.len(), db.len());
    assert!(seen_ids.len() >= 4000);

    // Look up popular headphone IDs
    let airpods_max = db.get_profile("apple-airpods-max-oratory1990");
    assert!(airpods_max.is_some(), "AirPods Max Oratory1990 profile must exist with unique ID");
    let apm = airpods_max.unwrap();
    assert_eq!(apm.brand, "Apple");
    assert_eq!(apm.model, "AirPods Max");
    assert!(apm.target_curve.contains("Harman Over-Ear"));

    // Multi-token query including target curve: "hd 600 oratory"
    let oratory_query = db.search("hd 600 oratory");
    assert!(!oratory_query.is_empty(), "Query 'hd 600 oratory' should match");
    assert_eq!(oratory_query[0].brand, "Sennheiser");
    assert!(oratory_query[0].target_curve.to_lowercase().contains("oratory"));
}
