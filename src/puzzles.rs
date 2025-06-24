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

    map.insert(73, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000001000000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"0000000000000000000000000000000000000000000001ffffffffffffffffff", 16).unwrap(),
    });

    map.insert(74, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000002000000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"0000000000000000000000000000000000000000000003ffffffffffffffffff", 16).unwrap(),
    });

    map.insert(76, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000008000000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"000000000000000000000000000000000000000000000fffffffffffffffffff", 16).unwrap(),
    });

    map.insert(77, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000010000000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"000000000000000000000000000000000000000000001fffffffffffffffffff", 16).unwrap(),
    });

    map.insert(78, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000020000000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"000000000000000000000000000000000000000000003fffffffffffffffffff", 16).unwrap(),
    });

    map.insert(79, PuzzleRange {
        start: BigUint::parse_bytes(b"0000000000000000000000000000000000000000000040000000000000000000", 16).unwrap(),
        end:   BigUint::parse_bytes(b"000000000000000000000000000000000000000000007fffffffffffffffffff", 16).unwrap(),
    });

    map
}
