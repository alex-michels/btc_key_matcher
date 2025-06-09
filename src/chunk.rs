use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

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

    pub fn save(&self, path: &str) {
        let file = File::create(path).expect("Unable to write chunk metadata file");
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).expect("Failed to serialize chunk metadata")
    }
}