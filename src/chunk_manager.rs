use crate::chunk::{ChunkMetadata, ChunkStatus, random_chunk_id, random_chunk_id_within_range};
use crate::puzzles::PuzzleRange;
use num_bigint::BigUint;
use std::fs;
use std::path::Path;

pub fn acquire_chunk(
    base_path: &str,
    chunk_size: &BigUint,
    cli_chunk_id: Option<BigUint>,
    puzzle_range: Option<&PuzzleRange>,
) -> (ChunkMetadata, BigUint) {
    let entries =
        fs::read_dir(base_path).unwrap_or_else(|_| panic!("Failed to read {}", base_path));

    if let Some(cli_id) = cli_chunk_id {
        let path = ChunkMetadata::path(&cli_id, base_path);
        if Path::new(&path).exists() {
            let mut chunk = ChunkMetadata::load(&path);
            match chunk.status {
                ChunkStatus::Pending => {
                    chunk.status = ChunkStatus::Processing;
                    chunk.save(&path);
                    return (chunk, cli_id);
                }
                _ => {
                    // Fallback to next available chunk
                }
            }
        } else {
            let chunk = ChunkMetadata::create_new(&cli_id, chunk_size, base_path, puzzle_range);
            return (chunk, cli_id);
        }
    }

    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(meta) = fs::read_to_string(&path) {
            if let Ok(mut chunk) = serde_json::from_str::<ChunkMetadata>(&meta) {
                if chunk.status == ChunkStatus::Pending {
                    let id =
                        BigUint::parse_bytes(chunk.chunk_id.trim_start_matches('0').as_bytes(), 10)
                            .expect("Invalid chunk ID");
                    chunk.status = ChunkStatus::Processing;
                    chunk.save(path.to_str().unwrap());
                    return (chunk, id);
                }
            }
        }
    }

    let new_id = if let Some(r) = puzzle_range {
        random_chunk_id_within_range(chunk_size, r)
    } else {
        random_chunk_id(chunk_size)
    };
    let chunk = ChunkMetadata::create_new(&new_id, chunk_size, base_path, puzzle_range);
    (chunk, new_id)
}
