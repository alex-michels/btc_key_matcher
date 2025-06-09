use btc_key_matcher::{
    chunk::ChunkMetadata,
    keygen::HexKeyGenerator,
    address::derive_addresses,
    search::{load_sorted_addresses, binary_search}
};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use hex;

#[test]
fn test_batch_scan_logic() {
    let meta = ChunkMetadata::load("resources/tests/test_chunk_not_found.json");
    let addresses = Arc::new(load_sorted_addresses("resources/tests/test_addresses_sorted.txt"));
    let mut generator = HexKeyGenerator::new(&meta.last_processed_hex, &meta.end_hex);

    let batch = generator.next_batch(10_000);
    let found = Arc::new(AtomicBool::new(false));

    batch.iter().for_each(|raw_key| {
        if found.load(Ordering::Relaxed) { return; }
        let derived = derive_addresses(raw_key);
        for addr in derived {
            if binary_search(&addresses, &addr) {
                found.store(true, Ordering::Relaxed);
                println!("[TEST] Found match: {} -> {}", hex::encode(raw_key), addr);
                break;
            }
        }
    });

    assert!(!found.load(Ordering::Relaxed), "Unexpected match found in this range");
}
