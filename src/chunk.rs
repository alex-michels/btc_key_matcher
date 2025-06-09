use serde::{Deserialize, Serialize};
use std::fs::{File, create_dir_all};
use std::io::{BufReader, BufWriter};
use num_bigint::BigUint;
use num_traits::identities::One;
use std::path::Path;

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