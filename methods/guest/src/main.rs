#![no_main]

use risc0_zkvm::{guest::env, serde};

use ark_bn254::{Bn254, G1Projective, G2Projective};
use ark_ec::{pairing::Pairing, Group};
use ark_serialize::CanonicalDeserialize;

risc0_zkvm::guest::entry!(main);
fn main() {
    // Fetch the input from the environment.
    // let start = env::cycle_count();

    let (pubkey_bytes, message_bytes, signature_bytes): (Vec<u8>, Vec<u8>, Vec<u8>) = env::read();
    let old_pubkey: Option<Vec<u8>> = env::read();
    let image_id: [u32; 8] = env::read::<[u32; 8]>();

    if let Some(pk) = old_pubkey {
        env::verify(image_id, &serde::to_vec(&pk).unwrap()).unwrap();
    }

    let pubkey = G2Projective::deserialize_compressed(&pubkey_bytes[..]).unwrap();
    let message = G1Projective::deserialize_compressed(&message_bytes[..]).unwrap();
    let signature = G1Projective::deserialize_compressed(&signature_bytes[..]).unwrap();

    let g2_gen: G2Projective = G2Projective::generator();

    // Pair
    let pairing_1 = Bn254::pairing(signature, g2_gen);
    let pairing_2 = Bn254::pairing(message, pubkey);

    assert_eq!(pairing_1, pairing_2);

    // let diff = env::cycle_count();
    // env::log(&format!("cycle count after BN254 verify: {}", diff - start));

    // Commit pubkey
    env::commit(&pubkey_bytes);
}
