use base58::ToBase58;
use bech32::{ToBase32, Variant, encode, u5};
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

fn sha256_digest(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

fn ripemd160_digest(data: &[u8]) -> Vec<u8> {
    Ripemd160::digest(data).to_vec()
}

fn hash160(data: &[u8]) -> Vec<u8> {
    ripemd160_digest(&sha256_digest(data))
}

fn to_p2pkh(pubkey: &[u8]) -> String {
    let hash = hash160(pubkey);
    let mut address = vec![0x00];
    address.extend(&hash);
    let checksum = &sha256_digest(&sha256_digest(&address))[..4];
    address.extend(checksum);
    address.to_base58()
}

fn to_p2sh(pubkey: &[u8]) -> String {
    let pubkey_hash = hash160(pubkey);
    let redeem_script: Vec<u8> = [&[0x00u8, 0x14][..], &pubkey_hash[..]].concat();
    let script_hash = hash160(&redeem_script);
    let mut address = vec![0x05];
    address.extend(&script_hash);
    let checksum = &sha256_digest(&sha256_digest(&address))[..4];
    address.extend(checksum);
    address.to_base58()
}

fn to_bech32(pubkey: &[u8]) -> String {
    let prog = hash160(pubkey);
    let mut bech32_data = vec![u5::try_from_u8(0).unwrap()];
    bech32_data.extend(prog.to_base32());
    encode("bc", bech32_data, Variant::Bech32).unwrap()
}

pub fn derive_addresses(raw_key: &[u8; 32]) -> Vec<String> {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(raw_key).unwrap();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    vec![
        to_p2pkh(&public_key.serialize()),
        to_p2pkh(&public_key.serialize_uncompressed()),
        to_p2sh(&public_key.serialize()),
        to_bech32(&public_key.serialize()),
    ]
}

pub fn private_key_to_wif(key: &[u8; 32], compressed: bool) -> String {
    let mut data = vec![0x80]; // Mainnet prefix
    data.extend_from_slice(key);
    if compressed {
        data.push(0x01);
    }
    let checksum = &Sha256::digest(&Sha256::digest(&data))[..4];
    data.extend(checksum);
    data.to_base58()
}
