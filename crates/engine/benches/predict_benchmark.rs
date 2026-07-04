use bytemuck::bytes_of;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs::File;
use std::io::Write;
use typeforge_common::dict_format::{AlphaIndex, DictionaryEntry, DictionaryHeader};
use typeforge_engine::engine::TypeForgeEngine;

fn setup_dummy_assets() -> (String, String, String) {
    let test_dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir_all(&test_dir).unwrap();

    let dict_path = test_dir.join("dict.bin").to_string_lossy().to_string();
    let l_db_path = test_dir.join("learning.db").to_string_lossy().to_string();
    let t_db_path = test_dir.join("telemetry.db").to_string_lossy().to_string();

    let mut file = File::create(&dict_path).unwrap();
    let mut header = DictionaryHeader::default();
    header.word_count = 10_000;

    let mut alpha: AlphaIndex = [0; 26];
    for i in 0..19 {
        alpha[i] = 0; // a-s have no words
    }
    alpha[19] = 0; // 't' starts at index 0, all words are testword{}
    for i in 20..26 {
        alpha[i] = 10_000; // u-z have no words
    }

    let mut entries = Vec::with_capacity(10_000);
    let mut pool = Vec::new();

    for i in 0..10_000 {
        let word = format!("testword{:04}", i); // testword0000 to testword9999 for proper sorting
        let entry = DictionaryEntry {
            offset: pool.len() as u32,
            length: word.len() as u16,
            first_char: b't' as u16,
            frequency: (10_000 - i) as u32,
        };
        entries.push(entry);
        pool.extend_from_slice(word.as_bytes());
    }

    header.index_offset = 48;
    header.strings_offset = 48 + 104 + (12 * 10_000);
    header.checksum_offset = header.strings_offset + pool.len() as u64;

    file.write_all(bytes_of(&header)).unwrap();
    file.write_all(bytemuck::cast_slice(&alpha)).unwrap();
    file.write_all(bytemuck::cast_slice(&entries)).unwrap();
    file.write_all(&pool).unwrap();
    file.write_all(&[0u8; 32]).unwrap(); // Dummy checksum

    (dict_path, l_db_path, t_db_path)
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
    let (dict_path, l_db_path, t_db_path) = setup_dummy_assets();
    let engine = TypeForgeEngine::new(dict_path, &l_db_path, &t_db_path, 5).unwrap();
    let req = dummy_req("testword50");

    c.bench_function("predict_th", |b| {
        b.iter(|| engine.predict(black_box("testword50"), &req, 5))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
