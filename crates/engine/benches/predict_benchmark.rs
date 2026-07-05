use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::path::PathBuf;
use typeforge_common::config::RankingConfig;
use typeforge_engine::engine::TypeForgeEngine;

fn get_real_dictionary_path() -> String {
    // Attempt to locate the real dictionary in the repo
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(manifest_dir);
    path.push("../../assets/dictionary.bin");

    if !path.exists() {
        // Fallback for when running from workspace root
        path = PathBuf::from("assets/dictionary.bin");
    }

    path.to_string_lossy().to_string()
}

fn setup_temp_dbs() -> (String, String) {
    let test_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&test_dir).unwrap();

    let l_db_path = test_dir.join("learning.db").to_string_lossy().to_string();
    let t_db_path = test_dir.join("telemetry.db").to_string_lossy().to_string();

    (l_db_path, t_db_path)
}

fn dummy_req(prefix: &str) -> typeforge_protocol::PredictRequest {
    typeforge_protocol::PredictRequest {
        prefix: prefix.to_string(),
        text_before_cursor: "".to_string(),
        text_after_cursor: "".to_string(),
        cursor_position: 0,
        application: None,
        language: None,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let dict_path = get_real_dictionary_path();
    let (l_db_path, t_db_path) = setup_temp_dbs();

    let ranking_config = RankingConfig::default();
    let engine = TypeForgeEngine::new(dict_path, &l_db_path, &t_db_path, ranking_config).unwrap();

    // Create a realistic request with text_before_cursor
    let mut req = dummy_req("prog");
    req.text_before_cursor = "We need a new prog".to_string();

    c.bench_function("predict_prog", |b| {
        b.iter(|| engine.predict(black_box("prog"), &req, 5))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
