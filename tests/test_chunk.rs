use btc_key_matcher::chunk::ChunkMetadata;
use std::fs;

#[test]
fn test_load_and_save_chunk_metadata() {
    let meta = ChunkMetadata {
        chunk_id: "test".to_string(),
        start_hex: "00".repeat(32),
        end_hex: "ff".repeat(32),
        last_processed_hex: "01".repeat(32),
    };

    let path = "resources/tests/tmp_test_chunk.json";
    meta.save(path);
    let loaded = ChunkMetadata::load(path);
    fs::remove_file(path).unwrap();

    assert_eq!(meta, loaded);
}
