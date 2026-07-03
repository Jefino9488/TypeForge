use criterion::{black_box, criterion_group, criterion_main, Criterion};
use typeforge_engine::engine::TypeForgeEngine;
use std::fs::File;
use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;

fn setup_dummy_assets() -> (String, String) {
    let test_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&test_dir).unwrap();
    
    let dict_path = test_dir.join("dict.csv.gz").to_string_lossy().to_string();
    let db_path = test_dir.join("test.db").to_string_lossy().to_string();
    
    let file = File::create(&dict_path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    // Create a reasonably sized dictionary for the benchmark
    for i in 0..10_000 {
        writeln!(encoder, "testword{},{}", i, 10000 - i).unwrap();
    }
    encoder.finish().unwrap();
    
    (dict_path, db_path)
}

fn criterion_benchmark(c: &mut Criterion) {
    let (dict_path, db_path) = setup_dummy_assets();
    let engine = TypeForgeEngine::new(dict_path, &db_path).unwrap();
    
    c.bench_function("predict_th", |b| {
        b.iter(|| engine.predict(black_box("testword50"), 5))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
