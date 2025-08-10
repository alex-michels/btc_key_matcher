use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn load_sorted_addresses(path: &str) -> Vec<String> {
    let file =
        File::open(path).unwrap_or_else(|e| panic!("Cannot open address file '{}': {:?}", path, e));
    let reader = BufReader::new(file);
    reader.lines().map(|l| l.unwrap()).collect()
}

pub fn binary_search(sorted: &[String], key: &str) -> bool {
    sorted.binary_search(&key.to_string()).is_ok()
}
