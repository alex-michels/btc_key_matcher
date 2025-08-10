use btc_key_matcher::keygen::HexKeyGenerator;

#[test]
fn test_keygen_batch_exact_size() {
    let start = "0000000000000000000000000000000000000000000000000000000000000000";
    let end = "0000000000000000000000000000000000000000000000000000000000000005";

    let mut generator = HexKeyGenerator::new(start, end);
    let batch = generator.next_batch(5);

    assert_eq!(batch.len(), 5);
    assert_eq!(hex::encode(batch[0]), start);
    assert_eq!(
        hex::encode(batch[4]),
        "0000000000000000000000000000000000000000000000000000000000000004"
    );
}

#[test]
fn test_keygen_stops_at_end() {
    let start = "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0";
    let end = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

    let mut generator = HexKeyGenerator::new(start, end);
    let batch = generator.next_batch(20); // request more than available
    assert_eq!(batch.len(), 16); // ✅ Fix: 16 keys from f0 to ff inclusive
    assert_eq!(hex::encode(batch.first().unwrap()), start);
    assert_eq!(hex::encode(batch.last().unwrap()), end);
}

#[test]
fn test_keygen_increment_logic() {
    let start = "00000000000000000000000000000000000000000000000000000000fffffffe";
    let end = "0000000000000000000000000000000000000000000000000000000100000005";

    let mut generator = HexKeyGenerator::new(start, end);
    let batch = generator.next_batch(10);

    assert_eq!(batch.len(), 8);
    assert_eq!(hex::encode(batch[0]), start);
    assert_eq!(
        hex::encode(batch[1]),
        "00000000000000000000000000000000000000000000000000000000ffffffff"
    );
    assert_eq!(
        hex::encode(batch[2]),
        "0000000000000000000000000000000000000000000000000000000100000000"
    );
    assert_eq!(
        hex::encode(batch.last().unwrap()),
        "0000000000000000000000000000000000000000000000000000000100000005"
    );
}

#[test]
fn test_keygen_single_key_range() {
    let start = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let end = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    let mut generator = HexKeyGenerator::new(start, end);
    let batch = generator.next_batch(5);

    assert_eq!(batch.len(), 1);
    assert_eq!(hex::encode(batch[0]), start);
}
