use btc_key_matcher::search::{binary_search, load_sorted_addresses};

#[test]
fn test_search_found_and_not_found() {
    let addresses = load_sorted_addresses("resources/tests/test_addresses_sorted.txt");

    assert!(binary_search(
        &addresses,
        "1KCohbCE8t97TRFT35szYC9srochLfzTs5"
    ));
    assert!(!binary_search(
        &addresses,
        "bc1noexistaddressxxxxxxxxxxxxxxxx"
    ));
}
