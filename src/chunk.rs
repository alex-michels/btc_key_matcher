use serde::{Deserialize, Serialize};
use std::fs::{self, File, create_dir_all};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use num_bigint::{BigUint, RandBigInt};
use num_traits::identities::One;
use rand::thread_rng;
use crate::puzzles::PuzzleRange;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkMetadata {
    pub chunk_id: String,
    pub start_hex: String,
    pub end_hex: String,
    pub last_processed_hex: String,
}

impl ChunkMetadata {
    /// Load an existing chunk metadata file from disk
    pub fn load(path: &str) -> Self {
        let file = File::open(path).expect("Unable to read chunk metadata file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Invalid chunk metadata JSON")
    }

    /// Save chunk metadata to disk
    pub fn save(&self, path: &str) {
        let file = File::create(path).expect("Unable to write chunk metadata file");
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).expect("Failed to serialize chunk metadata");
    }

    /// Load from file if it exists, or create new metadata based on chunk range and optional puzzle range
    pub fn load_or_create(
        chunk_id: &BigUint,
        chunk_size: &BigUint,
        base_path: &str,
        puzzle_range: Option<PuzzleRange>,
    ) -> Self {
        let padded_id = format!("{:0>5}", chunk_id.to_str_radix(10));
        let chunk_path = format!("{}/chunk_{}.json", base_path, padded_id);

        if Path::new(&chunk_path).exists() {
            Self::load(&chunk_path)
        } else {
            let (start_hex, end_hex) = if let Some(range) = puzzle_range {
                let base = &range.start + (chunk_id * chunk_size);
                let start = &base + BigUint::one(); // skip previous chunk's end
                let end = &start + chunk_size - BigUint::one();
                (
                    format!("{:064x}", start),
                    format!("{:064x}", end),
                )
            } else {
                calculate_chunk_range(chunk_id, chunk_size)
            };

            let meta = ChunkMetadata {
                chunk_id: padded_id.clone(),
                start_hex: start_hex.clone(),
                end_hex: end_hex.clone(),
                last_processed_hex: start_hex,
            };

            create_dir_all(base_path).expect("Failed to create chunk folder");
            meta.save(&chunk_path);
            meta
        }
    }
}

/// Return chunk file name like: `chunk_00042.json`
pub fn format_chunk_filename(chunk_id: &BigUint) -> String {
    format!("chunk_{}.json", chunk_id.to_str_radix(10))
}

/// Return the start and end hex of a chunk ID
pub fn calculate_chunk_range(chunk_id: &BigUint, chunk_size: &BigUint) -> (String, String) {
    let base = chunk_id * chunk_size;
    let start = &base + BigUint::one();
    let end = &start + chunk_size - BigUint::one();
    (
        format!("{:064x}", start),
        format!("{:064x}", end),
    )
}

/// Scan a folder for a JSON chunk file and return its ID
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

/// Generate a random chunk ID in the global keyspace
pub fn random_chunk_id(chunk_size: &BigUint) -> BigUint {
    let max_key = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
        16,
    ).unwrap();
    let mut rng = thread_rng();
    let max_chunks = &max_key / chunk_size;
    rng.gen_biguint_below(&max_chunks)
}

/// Generate a random chunk ID within a given puzzle range
pub fn random_chunk_id_within_range(chunk_size: &BigUint, range: &PuzzleRange) -> BigUint {
    let max_chunks = (&range.end - &range.start) / chunk_size;
    let mut rng = thread_rng();
    rng.gen_biguint_below(&max_chunks)
}
