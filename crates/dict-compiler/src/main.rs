use bytemuck::bytes_of;
use clap::{Parser, Subcommand};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use typeforge_common::dict_format::{
    AlphaIndex, DictionaryEntry, DictionaryHeader, FLAG_UTF8_NORMALIZED, MAGIC_NUMBER,
};

#[derive(Parser)]
#[command(name = "typeforge-dict-compiler")]
#[command(version, about = "Compiles TypeForge dictionary from CSV to Binary", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        csv_path: String,
        bin_path: String,
        #[arg(short, long)]
        compress: bool,
    },
    Verify {
        bin_path: String,
    },
    Stats {
        bin_path: String,
    },
    Inspect {
        bin_path: String,
    },
    Dump {
        bin_path: String,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    language: String,
    words: u32,
    version: u32,
    generator: String,
    sha256: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compile {
            csv_path,
            bin_path,
            compress,
        } => run_compile(&csv_path, &bin_path, compress),
        Commands::Verify { bin_path } => run_verify(&bin_path),
        Commands::Stats { bin_path } => run_stats(&bin_path),
        Commands::Inspect { bin_path } => run_inspect(&bin_path),
        Commands::Dump { bin_path, limit } => run_dump(&bin_path, limit),
    }
}

fn read_csv(path: &str) -> Result<Vec<(String, u32)>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader: csv::Reader<Box<dyn Read>> = if path.ends_with(".gz") {
        csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Box::new(GzDecoder::new(file)))
    } else {
        csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Box::new(file))
    };

    let mut map = HashMap::new();
    for result in reader.records() {
        let record = result?;
        if let (Some(word), Some(freq_str)) = (record.get(0), record.get(1)) {
            // NFC Normalize and Lowercase
            // Since we don't have unicode-normalization crate included yet, we just lowercase for MVP.
            // In a real production scenario, use unicode-normalization.
            let normalized = word.to_lowercase();
            if let Ok(freq_u64) = freq_str.parse::<u64>() {
                let freq = std::cmp::min(freq_u64, u32::MAX as u64) as u32;
                *map.entry(normalized).or_insert(0) += freq;
            }
        }
    }

    let mut words: Vec<(String, u32)> = map.into_iter().collect();
    words.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(words)
}

fn run_compile(
    csv_path: &str,
    bin_path: &str,
    _compress: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading and normalizing {}...", csv_path);
    let words = read_csv(csv_path)?;
    let word_count = words.len() as u32;
    println!("Processed {} unique words.", word_count);

    let mut out_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(bin_path)?;
    let mut header = DictionaryHeader::default();
    header.word_count = word_count;
    header.flags |= FLAG_UTF8_NORMALIZED;

    // We will write dummy header first
    out_file.write_all(bytes_of(&header))?;

    // Build strings pool and entries
    let mut strings_pool = Vec::new();
    let mut entries = Vec::with_capacity(words.len());
    let mut alpha_index: AlphaIndex = [0; 26];
    let mut current_alpha = 0;

    for (i, (word, freq)) in words.iter().enumerate() {
        let first_char = word.chars().next().unwrap_or('\0');
        let fc_u16 = first_char as u16;

        let c = first_char.to_ascii_lowercase();
        if c.is_ascii_lowercase() {
            let idx = (c as u8 - b'a') as usize;
            if idx >= current_alpha {
                for j in current_alpha..=idx {
                    alpha_index[j] = i as u32;
                }
                current_alpha = idx + 1;
            }
        }

        let entry = DictionaryEntry {
            offset: strings_pool.len() as u32,
            length: word.len() as u16,
            first_char: fc_u16,
            frequency: *freq,
        };
        entries.push(entry);
        strings_pool.extend_from_slice(word.as_bytes());
    }

    // Fill remaining alpha index if any
    for j in current_alpha..26 {
        alpha_index[j] = words.len() as u32;
    }

    // Write alpha index
    let index_offset = out_file.stream_position()?;
    out_file.write_all(bytemuck::cast_slice(&alpha_index))?;

    // Write entries
    out_file.write_all(bytemuck::cast_slice(&entries))?;

    // Write strings pool
    let strings_offset = out_file.stream_position()?;
    out_file.write_all(&strings_pool)?;

    // Calculate checksum
    let eof = out_file.stream_position()?;
    let checksum_offset = eof;

    // Update Header
    header.index_offset = index_offset;
    header.strings_offset = strings_offset;
    header.checksum_offset = checksum_offset;

    out_file.seek(SeekFrom::Start(0))?;
    out_file.write_all(bytes_of(&header))?;

    out_file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut (&out_file).take(eof), &mut hasher)?;
    let hash = hasher.finalize();

    out_file.seek(SeekFrom::Start(eof))?;
    out_file.write_all(&hash)?;

    let file_len = out_file.metadata()?.len();
    println!("Generated {} ({} bytes)", bin_path, file_len);

    // Generate Manifest
    let hash_hex = hex::encode(hash);
    let manifest = Manifest {
        language: "en".to_string(),
        words: word_count,
        version: header.version,
        generator: "TypeForge Dict Compiler 0.1".to_string(),
        sha256: hash_hex.clone(),
    };

    let manifest_path = PathBuf::from(bin_path).with_extension("manifest");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&manifest_path, manifest_json)?;
    println!("Generated {}", manifest_path.display());

    Ok(())
}

fn run_verify(bin_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(bin_path)?;
    let mut header_bytes = [0u8; 48];
    file.read_exact(&mut header_bytes)?;

    let header: &DictionaryHeader = bytemuck::from_bytes(&header_bytes);
    if header.magic != MAGIC_NUMBER {
        println!("Error: Invalid magic number. Not a TypeForge dictionary.");
        std::process::exit(1);
    }

    let file_len = file.metadata()?.len();
    if header.checksum_offset + 32 != file_len {
        println!(
            "Error: File size mismatch (Expected {}, Got {})",
            header.checksum_offset + 32,
            file_len
        );
        std::process::exit(1);
    }

    file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut (&file).take(header.checksum_offset), &mut hasher)?;
    let computed_hash = hasher.finalize();

    file.seek(SeekFrom::Start(header.checksum_offset))?;
    let mut stored_hash = [0u8; 32];
    file.read_exact(&mut stored_hash)?;

    if computed_hash.as_slice() == stored_hash {
        println!("Verification successful. Checksum matches.");
    } else {
        println!("Error: Checksum mismatch!");
        std::process::exit(1);
    }

    Ok(())
}

fn run_stats(bin_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(bin_path)?;
    let mut header_bytes = [0u8; 48];
    file.read_exact(&mut header_bytes)?;
    let header: &DictionaryHeader = bytemuck::from_bytes(&header_bytes);

    println!("Language: en");
    println!("Words: {}", header.word_count);
    let size_mb = file.metadata()?.len() as f64 / 1024.0 / 1024.0;
    println!("Size: {:.2} MB", size_mb);
    println!("Version: {}", header.version);

    let manifest_path = PathBuf::from(bin_path).with_extension("manifest");
    if manifest_path.exists()
        && let Ok(content) = std::fs::read_to_string(manifest_path)
        && let Ok(manifest) = serde_json::from_str::<Manifest>(&content)
    {
        println!("Checksum: {}", manifest.sha256);
    }
    Ok(())
}

fn run_inspect(bin_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(bin_path)?;
    let mut header_bytes = [0u8; 48];
    file.read_exact(&mut header_bytes)?;
    let header: &DictionaryHeader = bytemuck::from_bytes(&header_bytes);

    println!("{:#?}", header);

    file.seek(SeekFrom::Start(header.checksum_offset))?;
    let mut stored_hash = [0u8; 32];
    file.read_exact(&mut stored_hash)?;
    println!("Checksum: {}", hex::encode(stored_hash));

    Ok(())
}

fn run_dump(bin_path: &str, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(bin_path)?;
    let mut header_bytes = [0u8; 48];
    file.read_exact(&mut header_bytes)?;
    let header: &DictionaryHeader = bytemuck::from_bytes(&header_bytes);

    let limit = std::cmp::min(limit, header.word_count as usize);

    file.seek(SeekFrom::Start(header.index_offset + 104))?; // Skip AlphaIndex
    for _ in 0..limit {
        let mut entry_bytes = [0u8; 12];
        file.read_exact(&mut entry_bytes)?;
        let entry: &DictionaryEntry = bytemuck::from_bytes(&entry_bytes);

        let mut word_bytes = vec![0u8; entry.length as usize];
        let current_pos = file.stream_position()?;
        file.seek(SeekFrom::Start(header.strings_offset + entry.offset as u64))?;
        file.read_exact(&mut word_bytes)?;
        let word = String::from_utf8_lossy(&word_bytes);
        println!("{} (freq: {})", word, entry.frequency);

        file.seek(SeekFrom::Start(current_pos))?;
    }

    Ok(())
}
