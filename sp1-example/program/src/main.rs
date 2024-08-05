//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use ark_bn254::{Bn254, G1Projective, G2Projective};
use ark_ec::{pairing::Pairing, Group};
use ark_serialize::CanonicalDeserialize;
use sha2::{Digest, Sha256};

pub fn main() {
    let (pubkey_bytes, message_bytes, signature_bytes): (Vec<u8>, Vec<u8>, Vec<u8>) =
        sp1_zkvm::io::read();

    // Read the verification keys.
    let vkeys = sp1_zkvm::io::read::<Vec<[u32; 8]>>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<Vec<u8>>>();

    // Read pubkey_0
    let pubkey_0 = sp1_zkvm::io::read::<Vec<u8>>();

    assert_eq!(vkeys.len(), public_values.len());
    for i in 0..vkeys.len() {
        let deserialized: (Vec<u8>, Vec<u8>) = bincode::deserialize(&public_values[i]).unwrap();
        assert_eq!(pubkey_0, deserialized.0);

        let vkey = &vkeys[i];
        let public_values_digest = Sha256::digest(&public_values[i]);
        sp1_zkvm::lib::verify::verify_sp1_proof(vkey, &public_values_digest.into());
    }

    let pubkey = G2Projective::deserialize_compressed(&pubkey_bytes[..]).unwrap();
    let message = G1Projective::deserialize_compressed(&message_bytes[..]).unwrap();
    let signature = G1Projective::deserialize_compressed(&signature_bytes[..]).unwrap();

    let g2_gen: G2Projective = G2Projective::generator();

    // Pair
    let pairing_1 = Bn254::pairing(signature, g2_gen);
    let pairing_2 = Bn254::pairing(message, pubkey);

    assert_eq!(pairing_1, pairing_2);

    // Commit to the public values of the program.
    sp1_zkvm::io::commit(&(pubkey_0, pubkey_bytes));
}
