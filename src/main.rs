// src/main.rs
mod chunk;
mod keygen;
mod address;
mod search;

use chunk::ChunkMetadata;
use keygen::HexKeyGenerator;
use address::derive_addresses;
use search::{load_sorted_addresses, binary_search};

use rayon::prelude::*;
use sha2::{Sha256, Digest};
use base58::ToBase58;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use std::fs;

const BATCH_SIZE: usize = 10_000_000;
const PROGRESS_FILE: &str = "resources/tests/test_chunk_found.json";
const ADDR_FILE: &str = "addresses/Bitcoin_addresses_sorted.txt";

fn private_key_to_wif(key: &[u8; 32], compressed: bool) -> String {
    let mut data = vec![0x80]; // Mainnet prefix
    data.extend_from_slice(key);
    if compressed {
        data.push(0x01);
    }
    let checksum = &Sha256::digest(&Sha256::digest(&data))[..4];
    data.extend(checksum);
    data.to_base58()
}

fn main() {
    let mut meta = ChunkMetadata::load(PROGRESS_FILE);
    let sorted_addresses = Arc::new(load_sorted_addresses(ADDR_FILE));

    let mut generator = HexKeyGenerator::new(
        &meta.last_processed_hex,
        &meta.end_hex,
    );

    let start = Instant::now();
    loop {
        let batch = generator.next_batch(BATCH_SIZE);
        if batch.is_empty() {
            break;
        }

        let found = Arc::new(AtomicBool::new(false));
        let addresses = Arc::clone(&sorted_addresses);

        batch.par_iter().for_each(|raw_key| {
            if found.load(Ordering::Relaxed) { return; }

            let derived = derive_addresses(raw_key);
            for (i, addr) in derived.iter().enumerate() {
                if binary_search(&addresses, addr) {
                    let hex_key = hex::encode(raw_key);
                    let wif_uncompressed = private_key_to_wif(raw_key, false);
                    let wif_compressed = private_key_to_wif(raw_key, true);
                    let format = match i {
                        0 => "P2PKH compressed",
                        1 => "P2PKH uncompressed",
                        2 => "P2SH",
                        3 => "Bech32 (P2WPKH)",
                        _ => "Unknown",
                    };
                    println!("MATCH FOUND: {} -> {}", hex_key, addr);
                    let csv_data = format!(
                        "hex_key;matched_address;wif_uncompressed;wif_compressed;format\n{};{};{};{};{}\n",
                        hex_key,
                        addr,
                        wif_uncompressed,
                        wif_compressed,
                        format
                    );
                    fs::write("match_found.csv", csv_data).unwrap();
                    found.store(true, Ordering::Relaxed);
                    std::process::exit(0);
                }
            }
        });

        meta.last_processed_hex = generator.last_key();
        meta.save(PROGRESS_FILE);
        println!("Batch completed. Last processed key: {}", meta.last_processed_hex);
    }

    println!("Finished chunk in {:?}", start.elapsed());
}
