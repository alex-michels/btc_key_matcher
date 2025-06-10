use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all};
use std::io::{BufReader, BufWriter};
use num_traits::identities::One;
use std::path::Path;
use std::fs;
use rand::thread_rng;
use num_bigint::{BigUint, RandBigInt};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkMetadata {
    pub chunk_id: String,
    pub start_hex: String,
    pub end_hex: String,
    pub last_processed_hex: String,
}

impl ChunkMetadata {
    pub fn load(path: &str) -> Self {
        let file = File::open(path).expect("Unable to read chunk metadata file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Invalid chunk metadata JSON")
    }

    pub fn load_or_create(chunk_id: &BigUint, chunk_size: &BigUint, base_path: &str) -> Self {
        let chunk_id_str = chunk_id.to_str_radix(10);
        let padded_id = format!("{:0>5}", chunk_id_str);
        let chunk_path = format!("{}/chunk_{}.json", base_path, padded_id);

        if Path::new(&chunk_path).exists() {
            Self::load(&chunk_path)
        } else {
            let (start, end) = calculate_chunk_range(chunk_id, chunk_size);
            let meta = ChunkMetadata {
                chunk_id: padded_id.clone(),
                start_hex: start.clone(),
                end_hex: end.clone(),
                last_processed_hex: start,
            };
            create_dir_all(base_path).expect("Failed to create chunk folder");
            meta.save(&chunk_path);
            meta
        }
    }

    pub fn save(&self, path: &str) {
        let file = File::create(path).expect("Unable to write chunk metadata file");
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).expect("Failed to serialize chunk metadata");
    }
}

pub fn calculate_chunk_range(chunk_id: &BigUint, chunk_size: &BigUint) -> (String, String) {
    let base = chunk_id * chunk_size;
    let start = &base + BigUint::one(); // Skip last end of previous chunk
    let end = &start + chunk_size - BigUint::one();
    (
        format!("{:064x}", start),
        format!("{:064x}", end),
    )
}

use std::fmt::Write;

/// Format chunk file name for arbitrary BigUint values
pub fn format_chunk_filename(chunk_id: &BigUint) -> String {
    format!("chunk_{}.json", chunk_id.to_str_radix(10))
}

pub fn find_existing_chunk_id(folder: &str) -> Option<BigUint> {
    let path = Path::new(folder);
    if path.exists() {
        for entry in fs::read_dir(path).ok()? {
            let entry = entry.ok()?;
            let filename = entry.file_name().to_string_lossy().into_owned();
            if filename.starts_with("chunk_") && filename.ends_with(".json") {
                let id_str = filename
                    .strip_prefix("chunk_")?
                    .strip_suffix(".json")?
                    .trim_start_matches('0');
                if !id_str.is_empty() {
                    return BigUint::parse_bytes(id_str.as_bytes(), 10);
                }
            }
        }
    }
    None
}

pub fn random_chunk_id(chunk_size: &BigUint) -> BigUint {
    let max_key = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
        16,
    ).unwrap();
    let mut rng = thread_rng();
    let max_chunks = &max_key / chunk_size;
    rng.gen_biguint_below(&max_chunks)
}