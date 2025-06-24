use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use num_bigint::{BigUint, RandBigInt};
use num_traits::One;
use rand::thread_rng;
use crate::puzzles::PuzzleRange;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ChunkStatus {
    Pending,
    Processing,
    Finished,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ChunkMetadata {
    pub chunk_id: String,
    pub start_hex: String,
    pub end_hex: String,
    pub last_processed_hex: String,
    pub status: ChunkStatus,
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

    pub fn path(chunk_id: &BigUint, base_path: &str) -> String {
        format!("{}/{}", base_path, format_chunk_filename(chunk_id))
    }

    pub fn exists(chunk_id: &BigUint, base_path: &str) -> bool {
        Path::new(&Self::path(chunk_id, base_path)).exists()
    }

    pub fn create_new(
        chunk_id: &BigUint,
        chunk_size: &BigUint,
        base_path: &str,
        puzzle_range: Option<&PuzzleRange>,
    ) -> Self {
        let padded_id = format!("{:0>5}", chunk_id.to_str_radix(10));
        let (start_hex, end_hex) = if let Some(range) = puzzle_range {
            let base = &range.start + (chunk_id * chunk_size);
            let start = &base + BigUint::one();
            let end = &start + chunk_size - BigUint::one();
            (format!("{:064x}", start), format!("{:064x}", end))
        } else {
            calculate_chunk_range(chunk_id, chunk_size)
        };

        let meta = ChunkMetadata {
            chunk_id: padded_id,
            start_hex: start_hex.clone(),
            end_hex: end_hex.clone(),
            last_processed_hex: start_hex,
            status: ChunkStatus::Processing,
        };

        create_dir_all(base_path).expect("Failed to create chunk folder");
        meta.save(&Self::path(chunk_id, base_path));
        meta
    }
}

pub fn format_chunk_filename(chunk_id: &BigUint) -> String {
    format!("chunk_{}.json", chunk_id.to_str_radix(10))
}

/// Return the start and end hex of a chunk ID
pub fn calculate_chunk_range(chunk_id: &BigUint, chunk_size: &BigUint) -> (String, String) {
    let base = chunk_id * chunk_size;
    let start = &base + BigUint::one();
    let end = &start + chunk_size - BigUint::one();
    (format!("{:064x}", start), format!("{:064x}", end))
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

pub fn random_chunk_id_within_range(chunk_size: &BigUint, range: &PuzzleRange) -> BigUint {
    let max_chunks = (&range.end - &range.start) / chunk_size;
    let mut rng = thread_rng();
    rng.gen_biguint_below(&max_chunks)
}
