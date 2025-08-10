mod address;
mod chunk;
mod chunk_manager;
mod keygen;
mod puzzles;
mod search;

use address::{derive_addresses, private_key_to_wif};
use chunk::{ChunkMetadata, ChunkStatus};
use chunk_manager::acquire_chunk;
use keygen::HexKeyGenerator;
use search::{binary_search, load_sorted_addresses};

use ctrlc;
use num_bigint::BigUint;
use rayon::prelude::*;
use std::env;
use std::fs::{self};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

const BATCH_SIZE: usize = 5_000_000;
const ADDR_FILE: &str = "resources/addresses/Bitcoin_addresses_sorted.txt";
const CHUNK_FOLDER: &str = "resources/chunks";
const CHUNK_SIZE: &str = "100_000_000_000";

fn main() {
    let args: Vec<String> = env::args().collect();
    let chunk_size = BigUint::parse_bytes(CHUNK_SIZE.as_bytes(), 10).unwrap();

    let puzzle_id = args
        .iter()
        .position(|arg| arg == "--puzzle-id")
        .and_then(|i| args.get(i + 1))
        .and_then(|id| id.parse::<u32>().ok());

    let cli_chunk_id = args
        .iter()
        .position(|arg| arg == "--chunk-id")
        .and_then(|i| args.get(i + 1))
        .and_then(|id| BigUint::parse_bytes(id.as_bytes(), 10));

    let puzzle_range = puzzle_id.and_then(|pid| puzzles::get_puzzle_ranges().get(&pid).cloned());

    let base_folder = if let Some(pid) = puzzle_id {
        format!("{}/puzzle_{:03}", CHUNK_FOLDER, pid)
    } else {
        CHUNK_FOLDER.to_string()
    };

    let (mut meta, chunk_id) = acquire_chunk(
        &base_folder,
        &chunk_size,
        cli_chunk_id,
        puzzle_range.as_ref(),
    );

    // Set up Ctrl+C handler
    let meta_arc = Arc::new(Mutex::new(meta.clone()));
    let base_folder_clone = base_folder.clone();
    let chunk_id_clone = chunk_id.clone();
    {
        let meta_ctrlc = Arc::clone(&meta_arc);
        ctrlc::set_handler(move || {
            let mut meta = meta_ctrlc.lock().unwrap();
            meta.status = ChunkStatus::Pending;
            meta.save(&ChunkMetadata::path(&chunk_id_clone, &base_folder_clone));
            println!("\n🛑 Interrupted. Chunk status reset to pending.");
            std::process::exit(0);
        })
        .expect("Error setting Ctrl+C handler");
    }

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
        meta.save(&ChunkMetadata::path(&chunk_id, &base_folder));

        let elapsed = batch_start.elapsed();
        println!(
            "✅ Batch #{:03} completed in {:.2?}. Last key: {}\n",
            batch_counter, elapsed, meta.last_processed_hex
        );

        let mut shared = meta_arc.lock().unwrap();
        *shared = meta.clone(); // update shared state
    }

    meta.status = ChunkStatus::Finished;
    meta.save(&ChunkMetadata::path(&chunk_id, &base_folder));

    println!(
        "🏁 Finished chunk {} in {:.2?}",
        chunk_id,
        start_chunk_time.elapsed()
    );
}
