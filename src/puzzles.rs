use num_bigint::BigUint;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PuzzleRange {
    pub start: BigUint,
    pub end: BigUint,
}

pub fn get_puzzle_ranges() -> HashMap<u32, PuzzleRange> {
    let mut map = HashMap::new();

    map.insert(71, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000000400000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"00000000000000000000000000000000000000000000007fffffffffffffffff", 16).unwrap(),
    });

    map.insert(72, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000000800000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"0000000000000000000000000000000000000000000000ffffffffffffffffff", 16).unwrap(),
    });

    // Add more puzzle ranges as needed...

    map
}
