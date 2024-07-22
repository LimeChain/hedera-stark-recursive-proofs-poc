// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Generated crate containing the image ID and ELF binary of the build guest.
include!(concat!(env!("OUT_DIR"), "/methods.rs"));

use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use ark_bn254::{Fr, G1Projective, G2Projective};
use ark_ec::Group;
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;
use ark_std::UniformRand;
use sha2::{Digest, Sha384};

pub type SignatureInput = (Vec<u8>, Vec<u8>, Vec<u8>);

pub fn verify_signature(
    pk_old: Option<Vec<u8>>,
    receipt: Option<Receipt>,
    signature_input: SignatureInput,
) -> Receipt {
    let env = match receipt {
        Some(r) => ExecutorEnv::builder()
            .add_assumption(r)
            .write(&signature_input)
            .unwrap()
            .write(&pk_old)
            .unwrap()
            .write(&PROOFS_ID)
            .unwrap()
            .build()
            .unwrap(),
        None => ExecutorEnv::builder()
            .write(&signature_input)
            .unwrap()
            .write(&pk_old)
            .unwrap()
            .write(&PROOFS_ID)
            .unwrap()
            .build()
            .unwrap(),
    };

    let prover = default_prover();

    println!("Proving the signature");
    let proof = prover
        .prove(env, PROOFS_ELF)
        .unwrap();

    let receipt = proof.receipt;

    println!("Verifying the receipt");
    receipt.verify(PROOFS_ID).unwrap();

    let new_pubkey: Vec<u8> = receipt.journal.decode().unwrap();

    println!("Signature verified with pubkey: {:?}", new_pubkey);

    receipt
}

pub fn generate_inputs() -> Result<crate::SignatureInput, anyhow::Error> {
    let g1_gen: G1Projective = G1Projective::generator();
    let g2_gen: G2Projective = G2Projective::generator();

    let mut rng = ark_std::test_rng();
    let s1 = Fr::rand(&mut rng);
    let s2 = Fr::rand(&mut rng);

    let pk_old: G2Projective = g2_gen * s1;
    let pk_new: G2Projective = g2_gen * s2;

    let mut pk_old_bytes: Vec<u8> = Vec::new();
    pk_old.serialize_compressed(&mut pk_old_bytes).unwrap();

    let mut pk_new_bytes: Vec<u8> = Vec::new();
    pk_new.serialize_compressed(&mut pk_new_bytes).unwrap();

    let mut hasher = Sha384::new();
    hasher.update(pk_old_bytes);
    let message_hash = hasher.finalize();
    let message: G1Projective = g1_gen * Fr::from_le_bytes_mod_order(message_hash.as_slice());
    let mut message_bytes: Vec<u8> = Vec::new();
    message.serialize_compressed(&mut message_bytes).unwrap();

    let signature: G1Projective = message * s2;

    let mut signature_bytes: Vec<u8> = Vec::new();
    signature
        .serialize_compressed(&mut signature_bytes)
        .unwrap();

    Ok((pk_new_bytes, message_bytes, signature_bytes))
}


#[cfg(test)]
mod tests {
    use ark_bn254::{Fr, G1Projective, G2Projective};
    use ark_ec::Group;
    use ark_ff::PrimeField;
    use ark_serialize::CanonicalSerialize;
    use ark_std::UniformRand;
    use risc0_zkvm::Receipt;
    use sha2::{Digest, Sha384};
    use std::fs::File;
    use std::io::Write;

    fn generate_inputs() -> Result<crate::SignatureInput, anyhow::Error> {
        let g1_gen: G1Projective = G1Projective::generator();
        let g2_gen: G2Projective = G2Projective::generator();

        let mut rng = ark_std::test_rng();
        let s1 = Fr::rand(&mut rng);
        let s2 = Fr::rand(&mut rng);

        let pk_old: G2Projective = g2_gen * s1;
        let pk_new: G2Projective = g2_gen * s2;

        let mut pk_old_bytes: Vec<u8> = Vec::new();
        pk_old.serialize_compressed(&mut pk_old_bytes).unwrap();

        let mut pk_new_bytes: Vec<u8> = Vec::new();
        pk_new.serialize_compressed(&mut pk_new_bytes).unwrap();

        let mut hasher = Sha384::new();
        hasher.update(pk_old_bytes);
        let message_hash = hasher.finalize();
        let message: G1Projective = g1_gen * Fr::from_le_bytes_mod_order(message_hash.as_slice());
        let mut message_bytes: Vec<u8> = Vec::new();
        message.serialize_compressed(&mut message_bytes).unwrap();

        let signature: G1Projective = message * s2;

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature
            .serialize_compressed(&mut signature_bytes)
            .unwrap();

        Ok((pk_new_bytes, message_bytes, signature_bytes))
    }

    #[test]
    fn test_verify_signature() -> Result<(), anyhow::Error> {
        // read json file and parse it
        let receipt_bytes = std::fs::read_to_string("examples/receipt.json")?;

        let receipt: Receipt = serde_json::from_str(&receipt_bytes)?;

        let pk_old: Vec<u8> = receipt.journal.decode().unwrap();

        let g1_gen: G1Projective = G1Projective::generator();
        let g2_gen: G2Projective = G2Projective::generator();

        let mut rng = ark_std::test_rng();
        let s1 = Fr::rand(&mut rng);
        let pk_new: G2Projective = g2_gen * s1;
        let mut pk_new_bytes: Vec<u8> = Vec::new();
        pk_new.serialize_compressed(&mut pk_new_bytes).unwrap();

        let mut hasher = Sha384::new();
        hasher.update(&pk_old);
        let message_hash = hasher.finalize();
        let message: G1Projective = g1_gen * Fr::from_le_bytes_mod_order(message_hash.as_slice());
        let mut message_bytes: Vec<u8> = Vec::new();
        message.serialize_compressed(&mut message_bytes).unwrap();

        let signature: G1Projective = message * s1;

        let mut signature_bytes: Vec<u8> = Vec::new();
        signature
            .serialize_compressed(&mut signature_bytes)
            .unwrap();

        let signature_input = (pk_new_bytes, message_bytes, signature_bytes);

        super::verify_signature(Some(pk_old), Some(receipt), signature_input);

        Ok(())
    }

    #[test]
    #[ignore]
    fn generate_test_proof() -> Result<(), anyhow::Error> {
        let inputs = generate_inputs()?;

        let receipt: Receipt = super::verify_signature(None, None, inputs);

        let receipt_bytes = serde_json::to_string_pretty(&receipt).unwrap();
        std::fs::create_dir_all("./examples")?;
        let mut file = File::create("examples/receipt.json")?;

        // Write the serialized string to the file
        file.write_all(&receipt_bytes.as_bytes())?;

        Ok(())
    }

    // #[test]
    // #[ignore]
    // fn verify_groth16() -> Result<(), anyhow::Error> {
    //     let receipt_bytes = std::fs::read_to_string("examples/receipt.json")?;
    //     let receipt: Receipt = serde_json::from_str(&receipt_bytes)?;

    //     println!("Receipt: {:?}", receipt);

    //     let groth_16 = receipt.inner.groth16()?;

    //     println!("Groth16 proof: {:?}", groth_16);
    //     Ok(())
    // }
}
