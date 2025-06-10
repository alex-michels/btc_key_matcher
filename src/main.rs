mod chunk;
mod keygen;
mod address;
mod search;
mod puzzles;

use chunk::{ChunkMetadata, format_chunk_filename};
use keygen::HexKeyGenerator;
use address::derive_addresses;
use search::{load_sorted_addresses, binary_search};

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use base58::ToBase58;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Instant;
use std::fs::{self, create_dir_all};
use std::env;
use std::path::Path;
use num_bigint::{BigUint, RandBigInt};
use num_traits::One;
use rand::thread_rng;

const BATCH_SIZE: usize = 10_000_000;
const ADDR_FILE: &str = "resources/addresses/Bitcoin_addresses_sorted.txt";
const CHUNK_FOLDER: &str = "resources/chunks";
const CHUNK_SIZE: &str = "10000000000";

fn private_key_to_wif(key: &[u8; 32], compressed: bool) -> String {
    let mut data = vec![0x80];
    data.extend_from_slice(key);
    if compressed {
        data.push(0x01);
    }
    let checksum = &Sha256::digest(&Sha256::digest(&data))[..4];
    data.extend(checksum);
    data.to_base58()
}

fn find_existing_chunk_id(folder: &str) -> Option<BigUint> {
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

    let puzzle_id = args.iter()
        .position(|arg| arg == "--puzzle-id")
        .and_then(|i| args.get(i + 1))
        .and_then(|id| id.parse::<u32>().ok());

    let cli_chunk_id = args.iter()
        .position(|arg| arg == "--chunk-id")
        .and_then(|i| args.get(i + 1))
        .and_then(|id| BigUint::parse_bytes(id.as_bytes(), 10));

    let puzzle_range = puzzle_id
        .and_then(|pid| puzzles::get_puzzle_ranges().get(&pid).cloned());

    let base_folder = if let Some(pid) = puzzle_id {
        format!("{}/puzzle_{:03}", CHUNK_FOLDER, pid)
    } else {
        CHUNK_FOLDER.to_string()
    };

    let chunk_id: BigUint = if let Some(id) = cli_chunk_id {
        id
    } else if let Some(existing) = find_existing_chunk_id(&base_folder) {
        println!("🧠 Using existing chunk ID from disk: {}", existing);
        existing
    } else if let Some(range) = &puzzle_range {
        let max_chunks = (&range.end - &range.start) / &chunk_size;
        let mut rng = thread_rng();
        let id = rng.gen_biguint_below(&max_chunks);
        println!("🎲 Generated random chunk ID: {}", id);
        id
    } else {
        let random = random_chunk_id(&chunk_size);
        println!("🎲 No chunk ID provided/found. Generated random chunk ID: {}", random);
        random
    };

    let chunk_filename = format_chunk_filename(&chunk_id);
    let chunk_path = format!("{}/{}", base_folder, chunk_filename);

    let mut meta = if Path::new(&chunk_path).exists() {
        ChunkMetadata::load(&chunk_path)
    } else {
        let (start_hex, end_hex) = if let Some(range) = &puzzle_range {
            let base = &range.start + (&chunk_id * &chunk_size);
            let start = &base + BigUint::one();
            let end = &start + &chunk_size - BigUint::one();
            (
                format!("{:064x}", start),
                format!("{:064x}", end),
            )
        } else {
            chunk::calculate_chunk_range(&chunk_id, &chunk_size)
        };

        let meta = ChunkMetadata {
            chunk_id: format!("{:05}", chunk_id),
            start_hex: start_hex.clone(),
            end_hex: end_hex.clone(),
            last_processed_hex: start_hex.clone(),
        };

        create_dir_all(&base_folder).unwrap();
        meta.save(&chunk_path);
        meta
    };

    println!("\n🚀 Starting BTC Key Matcher");
    println!("➡️  Chunk ID: {}", meta.chunk_id);
    println!("   Start Key: {}", meta.start_hex);
    println!("   End Key:   {}", meta.end_hex);
    println!("   Last Key:  {}", meta.last_processed_hex);

    println!("📁 Loading address database from: {}", ADDR_FILE);
    let sorted_addresses = Arc::new(load_sorted_addresses(ADDR_FILE));
    println!("✅ Loaded {} addresses\n", sorted_addresses.len());

    let mut generator = HexKeyGenerator::new(&meta.last_processed_hex, &meta.end_hex);
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
            if found.load(Ordering::Relaxed) {
                return;
            }

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
