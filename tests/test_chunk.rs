use btc_key_matcher::chunk::{ChunkMetadata, ChunkStatus, calculate_chunk_range};
use num_bigint::BigUint;
use num_traits::{FromPrimitive, One}; // <== Use FromPrimitive instead of manual from()
use std::fs;

#[test]
fn test_calculate_chunk_range_boundaries() {
    let chunk_size = BigUint::from_u64(10_000_000_000).unwrap();

    // Chunk 0 should start at 1
    let chunk_id0 = BigUint::from_u64(0).unwrap();
    let (start0, end0) = calculate_chunk_range(&chunk_id0, &chunk_size);
    assert_eq!(
        start0,
        "0000000000000000000000000000000000000000000000000000000000000001"
    );
    assert_eq!(
        end0,
        "00000000000000000000000000000000000000000000000000000002540be400"
    );

    // Chunk 1
    let chunk_id1 = BigUint::from_u64(1).unwrap();
    let (start1, end1) = calculate_chunk_range(&chunk_id1, &chunk_size);
    assert_eq!(
        start1,
        "00000000000000000000000000000000000000000000000000000002540be401"
    );
    assert_eq!(
        end1,
        "00000000000000000000000000000000000000000000000000000004a817c800"
    );

    // Check that the chunk covering n-1 exists
    let n_minus_1 = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
        16,
    )
    .unwrap();

    let chunk_id_last = (&n_minus_1 - BigUint::one()) / &chunk_size;
    let (start_last, end_last) = calculate_chunk_range(&chunk_id_last, &chunk_size);

    let start_big = BigUint::parse_bytes(start_last.as_bytes(), 16).unwrap();
    let end_big = BigUint::parse_bytes(end_last.as_bytes(), 16).unwrap();

    assert!(start_big <= n_minus_1, "start does not reach n-1");
    assert!(end_big >= n_minus_1, "end does not cover n-1");
}

#[test]
fn test_chunk_metadata_file_persistence() {
    let path = "resources/tests/tmp_chunk_99999.json";

    let meta = ChunkMetadata {
        chunk_id: "99999".to_string(),
        start_hex: "0f".repeat(32),
        end_hex: "ff".repeat(32),
        last_processed_hex: "1f".repeat(32),
        status: ChunkStatus::Processing,
    };

    meta.save(path);
    let loaded = ChunkMetadata::load(path);
    fs::remove_file(path).unwrap();

    assert_eq!(meta, loaded);
}
