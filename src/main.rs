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
use std::env;
use std::path::Path;
use num_bigint::{BigUint, RandBigInt};
use num_traits::{Zero, One};
use rand::thread_rng;

const BATCH_SIZE: usize = 10_000_000;
const ADDR_FILE: &str = "resources/addresses/Bitcoin_addresses_sorted.txt";
const CHUNK_FOLDER: &str = "resources/chunks";
const CHUNK_SIZE: &str = "10000000000"; // As string to convert to BigUint easily

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

fn find_existing_chunk_id() -> Option<BigUint> {
    let path = Path::new(CHUNK_FOLDER);
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

fn random_chunk_id(chunk_size: &BigUint) -> BigUint {
    let max_key = BigUint::parse_bytes(
        b"fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
        16,
    ).unwrap();
    let mut rng = thread_rng();
    let max_chunks = &max_key / chunk_size;
    rng.gen_biguint_below(&max_chunks)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let chunk_size = BigUint::parse_bytes(CHUNK_SIZE.as_bytes(), 10).unwrap();

    // Determine chunk_id
    let chunk_id: BigUint = if args.len() > 1 {
        BigUint::parse_bytes(args[1].as_bytes(), 10).expect("Invalid chunk_id argument")
    } else if let Some(existing) = find_existing_chunk_id() {
        println!("🧠 Using existing chunk ID from disk: {}", existing);
        existing
    } else {
        let random = random_chunk_id(&chunk_size);
        println!("🎲 No chunk ID provided/found. Generated random chunk ID: {}", random);
        random
    };

    let chunk_path = format!("{}/chunk_{:05}.json", CHUNK_FOLDER, chunk_id);

    println!("\n🚀 Starting BTC Key Matcher");
    println!("📦 Loading chunk: {}", chunk_path);

    let mut meta = ChunkMetadata::load_or_create(&chunk_id, &chunk_size, CHUNK_FOLDER);
    println!("➡️  Chunk ID: {}", meta.chunk_id);
    println!("   Start Key: {}", meta.start_hex);
    println!("   End Key:   {}", meta.end_hex);
    println!("   Last Key:  {}", meta.last_processed_hex);

    println!("📁 Loading address database from: {}", ADDR_FILE);
    let sorted_addresses = Arc::new(load_sorted_addresses(ADDR_FILE));
    println!("✅ Loaded {} addresses\n", sorted_addresses.len());

    let mut generator = HexKeyGenerator::new(
        &meta.last_processed_hex,
        &meta.end_hex,
    );

    let start_chunk_time = Instant::now();
    let mut batch_counter = 0;

    loop {
        let batch = generator.next_batch(BATCH_SIZE);
        if batch.is_empty() {
            break;
        }

        let batch_start_hex = hex::encode(batch[0]);
        batch_counter += 1;
        println!(
            "🔁 Processing batch #{:03} | Start Key: {}",
            batch_counter, batch_start_hex
        );
        let batch_start = Instant::now();

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
                    println!("🎯 MATCH FOUND: {} -> {}", hex_key, addr);
                    let csv_data = format!(
                        "hex_key;matched_address;wif_uncompressed;wif_compressed;format\n{};{};{};{};{}\n",
                        hex_key, addr, wif_uncompressed, wif_compressed, format
                    );
                    fs::write("match_found.csv", csv_data).unwrap();
                    found.store(true, Ordering::Relaxed);
                    std::process::exit(0);
                }
            }
        });

        meta.last_processed_hex = generator.last_key();
        meta.save(&chunk_path);

        let elapsed = batch_start.elapsed();
        println!(
            "✅ Batch #{:03} completed in {:.2?}. Last key: {}\n",
            batch_counter, elapsed, meta.last_processed_hex
        );
    }

    println!("🏁 Finished chunk {} in {:.2?}", chunk_id, start_chunk_time.elapsed());
}
